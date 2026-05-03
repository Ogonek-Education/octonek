# Telegram GitHub Issue Bot

A Rust-based Telegram bot that allows authorized users to create GitHub issues using a conversational "User Story" wizard.

## Features

- **Conversational Wizard**: Guides you through creating an issue with a structured format.
- **Agile Template**: Automatically formats the issue body as a User Story:
  - **As a** [role]
  - **I need** [function]
  - **So that** [benefit]
  - **Details and Assumptions**
  - **Acceptance Criteria** (Gherkin style)
- **Authorization**: Restrict bot usage to specific Telegram User IDs.
- **Async**: Built with `teloxide` and `tokio` for high performance.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- A Telegram Bot Token (from [@BotFather](https://t.me/botfather))
- A GitHub Personal Access Token (PAT) with `repo` scope.

## Setup

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd ogonek-gh
   ```

2. **Configure Environment Variables**:
   Copy the example environment file and fill in your details:
   ```bash
   cp .env.example .env
   ```
   Edit `.env`:
   - `TELOXIDE_TOKEN`: Your Telegram bot token.
   - `GITHUB_PAT`: Your GitHub Personal Access Token.
   - `GITHUB_OWNER`: The owner (user or organization) of the target repo.
   - `GITHUB_REPO`: The name of the repository.
   - `ALLOWED_USER_IDS`: Comma-separated list of Telegram User IDs (e.g., `12345678,98765432`). Leave empty to allow anyone (not recommended).
   - `TARGET_CHAT_ID`: (Optional) Restrict the bot to a specific chat (group/channel). For private groups, prepend `-100` to the ID from the link (e.g., `t.me/c/3012627795/...` -> `-1003012627795`).
   - `TARGET_THREAD_ID`: (Optional) Restrict the bot to a specific Topic (Thread) within the chat.

3. **Find your Telegram User ID**:
   You can find your ID by messaging [@userinfobot](https://t.me/userinfobot) on Telegram.

## Usage

Run the bot using cargo:

```bash
cargo run --release
```

### Commands

- `/newissue` - Start the conversational wizard to create a new GitHub issue.
- `/cancel` - Abort the current issue creation process.
- `/help` - Show available commands.

### Conversational Flow

When you send `/newissue`, the bot will ask you for:
1. **Title**: The title of the GitHub issue.
2. **Role**: Who is this for? ("As a...")
3. **Function**: What is needed? ("I need...")
4. **Benefit**: Why is it needed? ("So that...")
5. **Details**: Any additional context or assumptions.
6. **Acceptance Criteria**: Gherkin-style criteria for completion.

Once finished, the bot will post the issue to GitHub and send you the link.

## Tech Stack

- **Framework**: [teloxide](https://github.com/teloxide/teloxide)
- **GitHub API**: [octocrab](https://github.com/XAMPPRocky/octocrab)
- **Runtime**: [tokio](https://github.com/tokio-rs/tokio)
