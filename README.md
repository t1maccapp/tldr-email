## How to run
0. Install rust `curl https://sh.rustup.rs -sSf | sh`
1. Obtain app password for [GMail](https://support.google.com/mail/answer/185833?hl=en), [Yandex Mail](https://yandex.com/support/mail/mail-clients/others.html) or any other with support of app passwords, IMAP & SMTP.
2. Run the CLI APP with one inbox
```
cargo run -- --account mail@inbox.com:pass 2> error.log;
```
3. Or run with multiple inboxes
```
cargo run -- --account mail@inbox.com:pass -- account mail2@inbox:pass 2> error.log;
```
4. Or run a bit safer
```
export PASS_1=$(pass mail@inbox.com); export PASS_2=$(pass mail2@inbox.com); cargo run -- --account mail@inbox.com:$PASS_1 --account mail2@inbox.com:"$PASS_2" 2> error.log;
```

## Known issues
1. Tls tunnel dies after some time of innactivity
