// Code based on information found at https://www.blinkingcaret.com/2017/11/29/asp-net-identity-passwordhash/

use std::convert::TryInto;
use byteorder::WriteBytesExt;
use std::io::{Write, BufRead};
use std::path::Path;

const SALT_LEN: usize = 16;
const HASH_LEN: usize = 32;

trait AspKDFDiscriminant {
    const ASP_KDF_DISCRIMINANT: u32;
}

impl AspKDFDiscriminant for hmac::Hmac<sha2::Sha256> {
    const ASP_KDF_DISCRIMINANT: u32 = 1;
}

impl AspKDFDiscriminant for hmac::Hmac<sha2::Sha512> {
    const ASP_KDF_DISCRIMINANT: u32 = 2;
}

fn encode_password<KDF: AspKDFDiscriminant + Clone + Sync + crypto_mac::Mac + crypto_mac::NewMac>(password: &str, rounds: u32) -> [u8; 1 + 4 + 4 + 4 + SALT_LEN + HASH_LEN] {
    let salt = rand::random::<[u8; SALT_LEN]>();
    let mut hash = [0u8; HASH_LEN];
    pbkdf2::pbkdf2::<KDF>(password.as_bytes(), &salt, rounds, &mut hash);

    let mut blob = [0u8; 1 + 4 + 4 + 4 + SALT_LEN + HASH_LEN];
    let mut cursor = &mut blob as &mut [u8];

    // Writing to memory can not fail
    // Identity version
    cursor.write_u8(1).unwrap();
    cursor.write_u32::<byteorder::BE>(KDF::ASP_KDF_DISCRIMINANT).unwrap();
    cursor.write_u32::<byteorder::BE>(rounds).unwrap();
    cursor.write_u32::<byteorder::BE>(SALT_LEN.try_into().expect("Salt longer than 2^32-1 B not supported")).unwrap();
    cursor.write_all(&salt).unwrap();
    cursor.write_all(&hash).unwrap();

    blob
}

fn read_password() -> String {
    println!("Enter your password and hit enter");
    println!("WARNING: you password will be visible - make sure nobody is looking over your shoulder!");

    let stdin = std::io::stdin();
    let mut stdin = std::io::BufReader::new(stdin.lock());
    let mut password = String::new();
    stdin.read_line(&mut password).expect("failed to read password");
    if password.ends_with('\n') {
        password.pop();
    }

    password
}

fn decode_btcpay_postgres_string(s: &str) -> postgres::Config {
    let mut result = postgres::Config::new();

    for part in s.split(';') {
        if part.is_empty() {
            continue;
        }
        let equals = part.find('=').expect("invalid part in btcpay config - missing equals (`=`) sign");
        let key = &part[0..equals];
        let val = &part[(equals + 1)..];

        match key {
            "User ID" => { result.user(val); },
            "Password" => { result.password(val); },
            "Host" => { result.host(val); },
            "Port" if !val.is_empty() => { result.port(val.parse().expect("invalid port number")); },
            "Port" => (),
            "Database" => { result.dbname(val); },
            unknown => eprintln!("Warning unknown key {} in btcpay config string", unknown),
        }
    }

    result
}

fn get_connection_details_from_btcpay_config(config: impl AsRef<Path>) -> postgres::Config {
    let file = std::io::BufReader::new(std::fs::File::open(config).expect("failed to open config file"));
    let postgres_key = "postgres=";

    for line in file.lines() {
        let line = line.expect("failed o read config file");

        if line.starts_with(postgres_key) {
            return decode_btcpay_postgres_string(&line[postgres_key.len()..]);
        }
    }

    panic!("postgres entry not found in the config");
}

fn main() {
    let mut args = std::env::args_os();
    args.next().expect("zeroth argument not given");
    let email = args.next().expect("Missing positional argument: email").into_string().expect("email is not UTF-8");
    let config = args.next().unwrap_or_else(|| "/etc/btcpayserver-system-mainnet/btcpayserver.conf".into());

    let postgres_config = get_connection_details_from_btcpay_config(&config);
    let password= read_password();
    // constant pulled from a live btcpay instance database
    let rounds = 10000;
    let encoded_password = encode_password::<hmac::Hmac<sha2::Sha256>>(&password, rounds);
    
    let mut connection = postgres_config.connect(postgres::NoTls).expect("Failed to connect to database");
    let affected = connection
        .execute("UPDATE \"AspNetUsers\" SET \"PasswordHash\" = encode($1, 'base64') WHERE \"Email\" = $2", &[&(&encoded_password as &[u8]) as &(dyn postgres::types::ToSql + Sync), &email])
        .expect("Failed to update database");

    match affected {
        0 => panic!("Invalid email"),
        1 => println!("Updating password succeeded"),
        n => panic!("Oh, shit, {} accounts were given the same password - duplicate emails?", n),
    }
}
