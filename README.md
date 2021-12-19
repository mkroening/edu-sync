# Edu Sync

[![CI](https://github.com/mkroening/edu-sync/actions/workflows/ci.yml/badge.svg)](https://github.com/mkroening/edu-sync/actions/workflows/ci.yml)

Edu Sync is a command line application that synchronizes the contents of Moodle instances to your computer.

It accesses the Moodle instance via the Moodle mobile web services API, which is also used by the official Moodle mobile app.
To be able to use Edu Sync with a Moodle instance, the Moodle mobile web services must explicitly be enabled by the instance.

This application is written in Rust with a focus on speed.
Downloads are performed concurrently, which is beneficial when syncing many small files.

## Usage

You can view more detailed help information with:

```bash
$ edu-sync-cli help
```

1.  Add an account:

    *   Using username and password:

        ```bash
        $ edu-sync-cli add --username <username> https://example.com ~/download-dir
        # You will be prompted to enter your password
        ```

    *   Using a token for the Moodle mobile web service — some instances show it in preferences/security keys:

        ```bash
        $ edu-sync-cli add https://example.com ~/download-dir
        # You will be prompted to enter your token
        ```

2.  Fetch available courses (populates the config file with courses):

    ```bash
    $ edu-sync-cli fetch
    ```

3.  Configure which courses to sync in the config file. Get the config path with:

    ```bash
    $ edu-sync-cli config
    ```

4.  Sync:

    ```bash
    $ edu-sync-cli sync
    ```

## Installation

The binary name for Edu Sync is `edu-sync-cli`.

[Archives of precompiled binaries](https://github.com/mkroening/edu-sync/releases) for Edu Sync are available for Windows, macOS and Linux.

If you're an Arch Linux user, then you can install Edu Sync from the [Arch User Repository](https://aur.archlinux.org/packages/edu-sync/).

## Licensing

This project is licensed under
* The GNU General Public License v3.0 only ([LICENSE](LICENSE), or https://www.gnu.org/licenses/gpl-3.0.html)

### Trademark Notice

Moodle™ is a [registered trademark](https://moodle.com/trademarks/) of Moodle Pty Ltd in many countries. Edu Sync is not sponsored, endorsed, licensed by, or affiliated with Moodle Pty Ltd.
