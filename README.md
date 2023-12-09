# imap_sorter

Automatically sort email using a connection to a mailbox

## goal

Having an independant program to sort a mailbox, whithout relying on a email client like outlook, thunderbird or eM client.

It can be run on a regular basis an sort your inbox (at the moment only sort email in mailbox, but it is easy to change this beaviour.

## How to

1. run it once `cargo run`  
2. edit a config.ini file, with server, username, password

   ```{json}
   imap_server = 'localhost'
   imap_port = 993
   imap_username = 'user'
   imap_password = ''
   ```

  if you want to secure your password using secure store, please see further down

4. create a rules.yaml file with all your rules

   ```{yaml}
   rules:
    - name: title
      filter: "FROM example@example.com"
      target: "targe_folder"
      enable: True
   ```

6. run it

## Filters

filter in rules are simply the one described in rfc3501, see https://datatracker.ietf.org/doc/html/rfc3501#section-6.4.4

### example

* all mail from <john@doe.com>
     `FROM john@doe.com`
* all mail from john and jane
     `OR FROM john@doe.com FROM jane@doe.com`
* all mail from John with suject Sport
     `FROM john@doe.com SUBJECT Sport`

## Installation

on windows need openssl.

```{shell}


    clone vcpkg https://github.com/Microsoft/vcpkg
    open directory where you've cloned vcpkg
    run ./bootstrap-vcpkg.bat
    run ./vcpkg.exe install openssl-windows:x64-windows
    run ./vcpkg.exe install openssl:x64-windows-static
    run ./vcpkg.exe integrate install
    run set VCPKGRS_DYNAMIC=1 (or simply set it as your environment variable)
      or $env:VCPKGRS_DYNAMIC=1 (in powershell)

```

## Using secure store

You need to use ssclient to create and update seurestore, here is how

```{shell}
cargo install ssclient
## you will need to enter a password twice, choose a long password, you will need it twice for create and once more to create the secrets.key file
ssclient create config.json
ssclient --export-key secrets.key -s .\config.json add imap_username 
ssclient -k .\secrets.key -s .\config.json set imap_password
ssclient -k .\secrets.key -s .\config.json set imap_hostname
```

do not forget to remove those three values from config.ini file
