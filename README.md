# Atlas

Atlas is a geoip HTTP service built in Rust, designed to simplify IP GeoLocation by automatically
managing MaxMind databases. It ensures that users have up-to-date IP data without the hassle of
manual downloads and updates. This tool is perfect for services who need reliable and fast
IP GeoLocation lookup in their applications using a very simple HTTP API.

## Features

- Automatic Database Updates: Automatically downloads and updates MaxMind's GeoIP databases.
- Dead-simple HTTP API: Atals has a very simple HTTP API that users can use and query IP Data.
- Efficient and Low Memory Footprint: Built with Rust for safety and performance also to minimize memory footprint during database updates and normal operations.
- Configurable: Allows users to select which edition of the MaxMind databases they want to use (e.g., GeoLite2, GeoIP2, Enterprise, ISP).

## Running Atlas

To run Atlas, you can choose one of the methods below.

### Using Docker (Recommended)

[TODO: Add docker instruction]

### Compiling from source

Firstly, make sure you have rustc and cargo installed. Then after cloning the repository you can
simply run the following command to build a release binary to use in your service.

```
cargo build --release
```

You can find the release binaries at `target/release/atlas`. Then you can run atlas by running
`./target/release/atlas` in your terminal. Make sure that the `DB_PATH` directory already exists
otherwise atlas crashes on startup.

## Configuration

Atlas uses OS environment variables for configuration. Here are the list of environment variables
that atlas looks into.

- MAXMIND_ACCOUNT_ID: Your Maxmind Account ID used to download the database. **Required**
- MAXMIND_LICENSE_KEY: Your Maxmind license key used to download the database. **Required**
- MAXMIND_DB_VARIANT: (Also called Edition ID) The database edition to used. Default is `GeoLite2-City`.
- MAXMIND_DB_DOWNLOAD_URL: Database download URL (only change if your download URL differs). Default is `https://download.maxmind.com/geoip/databases/{VARIANT}/download?suffix=tar.gz`.

- DB_PATH: Default path to save databases in. Default is `/opt/atlas/db`.
- DB_UPDATE_INTERVAL_SECONDS: How often to check for updates in seconds. Default is a day (86400s).
- HOST: Host to serve Atlas API on. Default is `0.0.0.0`.
- PORT: Port number to serve Atlas API on. Default is `8080`.
- SWAGGER_UI_ENABLED: If set to `true` swagger UI will be served on `http://{HOST}:{PORT}/swagger-ui` endpoint. Default is `false`.

## Contribution

Contributions to Atlas are very welcome! Before undertaking any substantial work, please consider
opening an issue to discuss ideas and planned approaches so we can ensure we keep progress moving
in the same direction. Always remember to be respectful of others.

## License

[Apache License 2.0](/LICENSE)