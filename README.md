```text

███╗   ██╗███████╗███████╗████████╗███████╗ ██████╗ ██████╗  ██████╗ ███████╗
████╗  ██║██╔════╝██╔════╝╚══██╔══╝██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
██╔██╗ ██║█████╗  ███████╗   ██║   █████╗  ██║   ██║██████╔╝██║  ███╗█████╗
██║╚██╗██║██╔══╝  ╚════██║   ██║   ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
██║ ╚████║███████╗███████║   ██║   ██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚═╝  ╚═══╝╚══════╝╚══════╝   ╚═╝   ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝

```

# 🦀 NestForge

**The performance of Rust. The structure of NestJS.**

NestForge is a high-performance backend framework designed for developers who crave the modularity and **Dependency Injection (DI)** of NestJS but want the memory safety and blazing speed of the Rust ecosystem.

> [!IMPORTANT]
> **Stable Release**
> NestForge **1.0.0** is now published on crates.io.

## ✨ Features

- 🚀 **Blazing Fast**: Built on top of **Axum** and **Tokio**, NestForge delivers top-tier performance out of the box.
- 🧩 **Modular Architecture**: Organize your code into **Modules**, **Controllers**, and **Services** for better maintainability.
- 💉 **Dependency Injection**: A powerful DI system to manage your application's dependencies.
- 🛡️ **Type Safety**: Leverage Rust's type system to catch errors at compile time.
- 🌐 **HTTP First**: A robust HTTP layer with support for routing, middleware, and more.

## 🚀 Getting Started

```bash
# Ensure Cargo is installed before you run these commands.
git clone https://github.com/vernonthedev/nestforge.git
cd nestforge
cargo check
cargo run -p hello-nestforge
```

You should be able to see the following output:

```text
🦀 NestForge running on http://[IP_ADDRESS]
```

## 🗃️ NestForge CLI Setup

```bash
# Install CLI
cargo install nestforge-cli

# Create a new NestForge application
nestforge new demo-api
cd demo-api
cargo run

# Generate a new resource (inside app folder)
nestforge g resource users
```

## Use NestForge In Your App

```toml
[dependencies]
nestforge = "1.0.0"
```

## Documentation

- Wiki: [https://github.com/vernonthedev/nestforge/wiki](https://github.com/vernonthedev/nestforge/wiki)
- Project docs: [https://vernonthedev.github.io/nestforge/docs/Home.md](https://vernonthedev.github.io/nestforge/docs/Home.md)

## 🤝 Contributing

Contributions are welcome!
For support or questions, please open an issue.

## 🙏 Acknowledgments

- **NestJS**: For the inspiration and architectural patterns.
- **Axum**: For the high-performance HTTP framework.
- **Tokio**: For the asynchronous runtime.
- **Tower**: For the middleware ecosystem.
