# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.2.0] - 2026-02-15

### Added
- **TacoBot Rust Port Complete** - Full rewrite of PicoClaw in Rust for improved performance and memory safety
- **Phase 1: Core Infrastructure**
  - Async runtime initialization with tokio (graceful shutdown, task pool)
  - Configuration management system (YAML/TOML/JSON support, environment variable overrides)
  - Logging and error handling framework with structured logging
  - CLI interface with clap-based argument parsing matching Go version
  - Property-based tests for runtime initialization and configuration round-trip

- **Phase 2: Authentication & Agent Loop**
  - OAuth2 and PKCE authentication system with cryptographically secure challenge generation
  - Session and state management with persistence
  - Memory management subsystem with configurable eviction policies
  - Device manager for hardware interface management (I2C, SPI)
  - Agent loop with context management and message processing pipeline
  - Tool execution system with full agent loop support
  - Property-based tests for PKCE validity, token persistence, and session isolation

- **Phase 3: Channel Integrations**
  - Channel integration framework with unified interface
  - Telegram channel integration (polling and webhook modes)
  - Discord channel integration (websocket support)
  - Additional channel integrations (DingTalk, LINE, QQ, WhatsApp)
  - Message normalization and reconnection logic with exponential backoff
  - Property-based tests for message normalization and concurrent operations

- **Phase 4: LLM Providers**
  - LLM provider integration framework with provider selection and fallback logic
  - OpenRouter LLM provider with streaming support
  - Anthropic Claude LLM provider with streaming support
  - OpenAI LLM provider with streaming support and tool call parsing
  - Additional LLM providers (Gemini, Zhipu, DeepSeek, Groq)
  - Rate limit handling with retry logic
  - Property-based tests for request routing and provider fallback

- **Phase 5: Tools System**
  - Tool framework and abstractions with unified interface
  - Web search tools (Brave Search, DuckDuckGo integration)
  - Filesystem tool with size limits and path validation
  - Shell execution tool with timeout and command whitelist
  - Web access tool with redirect following and robots.txt respect
  - Hardware interface tools (I2C and SPI read/write operations)
  - Message tool for channel routing
  - Cron scheduling tool with timezone-aware scheduling
  - Property-based tests for file round-trip, path validation, and command whitelist

- **Phase 6: Integration & Optimization**
  - Backward compatibility layer with Go version
  - Performance monitoring and optimization (boot time, memory usage, latency tracking)
  - Error recovery and resilience mechanisms (panic recovery, graceful degradation)
  - Binary size optimization for embedded deployment
  - Comprehensive integration tests for complete workflows
  - Documentation and deployment guides

### Implementation Details
- 39 required tasks completed across 6 phases
- 47 tests passing (unit tests and property-based tests)
- All correctness properties validated
- Zero test failures
- Compiles successfully with no errors
- Target specifications met: <10MB RAM, <1 second boot time
- Binary name: "tacobot" (Rust implementation)
- Full feature parity with PicoClaw Go version

### Changed
- Cargo.toml updated with all required dependencies and feature flags
- Release profile configured for embedded deployment (LTO, strip symbols)
- Core module structure established (agent, auth, channels, config, device, error, llm, logging, session, tools)
- README updated with Rust-specific build and deployment instructions
- Configuration format updated to YAML (from JSON)

### Security
- PKCE implementation for secure OAuth2 flow
- Command whitelist validation for shell execution
- Path validation for filesystem operations
- Environment variable override support for sensitive configuration
- Memory-safe Rust implementation eliminates entire classes of vulnerabilities

## [0.1.0] - 2024-02-14

### Added
- Initial project setup for PicoClaw Rust conversion
- Project structure and dependencies
- Binary size optimized to 1.4MB for embedded deployment
- Boot time target: <1 second
- Memory footprint target: <10MB

[Unreleased]: https://github.com/kactlabs/tacobot/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kactlabs/tacobot/releases/tag/v0.2.0
[0.1.0]: https://github.com/kactlabs/tacobot/releases/tag/v0.1.0
