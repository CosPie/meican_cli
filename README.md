# Meican CLI

Meican (美餐) CLI is a fast and easy-to-use command-line interface written in Rust to order meals from the Meican platform directly from your terminal.

## Features
- **Account Management**: Seamlessly login, securely save your session, and manage your account.
- **Menu & Calendar**: View today's meal slots, check the calendar, and browse available dishes and restaurants.
- **Order Management**: Order dishes, check your delivery addresses, or cancel an existing order.
- **History**: View your past order history.
- **Flexible Output**: Supports raw JSON data or well-formatted tables (`--table`).
- **Self-Updating**: Keep your CLI up to date with a built-in update command.

## Installation

### Using the Install Script
You can quickly install or update the latest version via curl:
```bash
curl -fsSL https://raw.githubusercontent.com/CosPie/meican_cli/master/install.sh | bash
```

Or if you have already cloned the repository:
```bash
./install.sh
```

### Building from Source (Cargo)
If you have Rust and Cargo installed, you can build and install it locally:

```bash
git clone https://github.com/yourusername/meican_cli.git
cd meican_cli
cargo install --path .
```

## Usage

Use the `--table` global flag with any command to view the output formatted as a table rather than JSON.

### Authentication

**Login to Meican** (will prompt for password if not provided):
```bash
meican login your.email@example.com
```

**Check login status**:
```bash
meican status
```

**Logout**:
```bash
meican logout
```

### Browsing Meals & Menus

**Show today's meal slots and orders**:
```bash
meican --table today
```

**Show calendar for a date range**:
```bash
meican --table calendar 2023-10-01 2023-10-07
```

**List available dishes or restaurants**:
```bash
meican --table dishes breakfast
meican --table restaurants lunch
```

### Managing Orders

**Place an order**:
```bash
# Order a specific dish ID for a meal slot
meican order lunch --dish <DISH_ID>
```

**Cancel an order**:
```bash
meican cancel lunch
# or cancel by order ID
meican cancel --id <ORDER_ID>
```

**List delivery addresses**:
```bash
meican --table addresses
```

**Show order history**:
```bash
# Show history for the last 30 days
meican --table history --days 30
```

### Updating
Update your `meican` CLI tool to the latest version:
```bash
meican update
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to follow the rust ecosystem's formatting guidelines (`cargo fmt`) and check for lint errors (`cargo clippy`).

## License

[MIT](https://choosealicense.com/licenses/mit/)
