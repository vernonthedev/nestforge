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
> **Beta Software**
> 🧪 NestForge is currently in **Beta**. The API is not yet stable and may undergo breaking changes before the 1.0.0 release. It is not recommended for production use quite yet.

## ✨ Features

- 🚀 **Blazing Fast**: Built on top of **Axum** and **Tokio**, NestForge delivers top-tier performance out of the box.
- 🧩 **Modular Architecture**: Organize your code into **Modules**, **Controllers**, and **Services** for better maintainability.
- 💉 **Dependency Injection**: A powerful DI system to manage your application's dependencies.
- 🛡️ **Type Safety**: Leverage Rust's type system to catch errors at compile time.
- 🌐 **HTTP First**: A robust HTTP layer with support for routing, middleware, and more.

## 🚀 Getting Started

```bash
# Ensure Cargo is installed before you run these commands.
cargo check  # Check for any linting errors and warnings.
cargo run   # Run the application in development mode.
```

You should be able to see the following output:

```text
🦀 NestForge running on http://[IP_ADDRESS]
```

## 🗃️ NestForge CLI Setup

```bash
# Create a new NestForge application in the examples directory.
cargo run -p nestforge-cli -- new demo-api
# Generate a new resource.
cargo run --manifest-path ../../crates/nestforge-cli/Cargo.toml -- g resource users
```

## 🤝 Contributing

Contributions are welcome!
For support or questions, please open an issue.

## 🙏 Acknowledgments

- **NestJS**: For the inspiration and architectural patterns.
- **Axum**: For the high-performance HTTP framework.
- **Tokio**: For the asynchronous runtime.
- **Tower**: For the middleware ecosystem.
