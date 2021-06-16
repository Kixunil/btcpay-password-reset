# BTCPay password reset

A simple tool for resetting BTCPayServer password more sanely.

## About

The official btcpayserver-docker contains a password reset tool that is not that great - you have to create a second admin account.
I wanted something saner - setting the desired password for specific account.
The reason this is not in C# is I needed it quickly and couldn't afford to wait for someone else to write it, so I wrote it in the language I know the best - Rust.

Feel free to translate it to C# or whatever is more suitable for inclusion in BTCPayServer.
The code is based on [the excellent article explaining how passwords are encoded by ASP.NET](https://www.blinkingcaret.com/2017/11/29/asp-net-identity-passwordhash/)

## Building

0. install Rust (`apt install cargo` on Debian, see https://rustup.rs for other platforms)
1. Run `cargo build` in the repository top-level directory
2. You will find the binary in `./target/debug/btcpay-password-reset`

If you're packging this tool, please use `cargo build --release` to create a smaller package.
You will find the release binary in `./target/release/btcpay-password-reset`

## Usage

`btcpay-password-reset email@domain [/path/to/btcpay_config/file]`

Enter your password and hit enter.
Beware, the password is visible!

If the path to config file is not specified the default from [Cryptoanarchy Debian Repoitory](https://github.com/debian-cryptoanarchy/cryptoanarchy-deb-repo-builder) mainnet is used (`/etc/btcpayserver-system-mainnet/btcpayserver.conf`).

## License

WTFPL, just please keep the link to the article explaining how ASP.NET passwords work.
It helped me a lot so I want to be nice to the author. :)
