use crate::github;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
    types::{KeyboardButton, KeyboardMarkup, KeyboardRemove},
    utils::command::BotCommands,
};
use tracing::{info, warn, instrument};

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, Debug)]
pub struct IssueData {
    pub title: String,
    pub issue_type: String,
    pub labels: Vec<String>,
    pub role: String,
    pub function: String,
    pub benefit: String,
    pub details: String,
    pub acceptance_criteria: String,
}

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    Start,
    ReceiveTitle,
    ReceiveIssueType {
        data: IssueData,
    },
    ReceiveLabels {
        data: IssueData,
    },
    ReceiveRole {
        data: IssueData,
    },
    ReceiveFunction {
        data: IssueData,
    },
    ReceiveBenefit {
        data: IssueData,
    },
    ReceiveDetails {
        data: IssueData,
    },
    ReceiveAcceptanceCriteria {
        data: IssueData,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start creating a new issue.")]
    NewIssue,
    #[command(description = "cancel the current operation.")]
    Cancel,
}

#[instrument(skip(bot, dialogue), fields(chat_id = %msg.chat.id, user = ?msg.from))]
pub async fn start(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    if !is_authorized(&msg) {
        warn!("Unauthorized access attempt");
        bot.send_message(msg.chat.id, "Unauthorized.").await?;
        return Ok(());
    }
    info!("Bot started");
    bot.send_message(msg.chat.id, "Send /newissue to start creating a GitHub issue.").await?;
    dialogue.exit().await?;
    Ok(())
}

#[instrument(skip(bot))]
pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

#[instrument(skip(bot, dialogue))]
pub async fn cancel(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    info!("Operation cancelled");
    bot.send_message(msg.chat.id, "Cancelled.")
        .reply_markup(KeyboardRemove::new())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

#[instrument(skip(bot, dialogue), fields(chat_id = %msg.chat.id, user = ?msg.from))]
pub async fn new_issue(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    if !is_authorized(&msg) {
        warn!("Unauthorized /newissue attempt");
        bot.send_message(msg.chat.id, "Unauthorized.").await?;
        return Ok(());
    }
    info!("Starting new issue wizard");
    bot.send_message(msg.chat.id, "What is the title of the issue?").await?;
    dialogue.update(State::ReceiveTitle).await?;
    Ok(())
}

#[instrument(skip(bot, dialogue))]
pub async fn receive_title(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    let title = msg.text().unwrap_or_default().to_string();
    info!(title = %title, "Received title");
    let data = IssueData {
        title,
        ..Default::default()
    };
    
    let types = vec!["Bug", "Feature", "Enhancement", "Documentation", "Task"];
    let keyboard = make_keyboard(types, 2);

    bot.send_message(msg.chat.id, "Select Issue Type:")
        .reply_markup(keyboard)
        .await?;
    
    dialogue.update(State::ReceiveIssueType { data }).await?;
    Ok(())
}

#[instrument(skip(bot, dialogue, data))]
pub async fn receive_issue_type(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    let issue_type = msg.text().unwrap_or_default().to_string();
    let valid_types = vec!["Bug", "Feature", "Enhancement", "Documentation", "Task"];
    
    if !valid_types.contains(&issue_type.as_str()) {
        bot.send_message(msg.chat.id, "Invalid type. Please select from the keyboard.").await?;
        return Ok(());
    }

    info!(issue_type = %issue_type, "Received issue type");
    data.issue_type = issue_type;

    // Fetch labels from GitHub
    match github::get_labels().await {
        Ok(labels) => {
            let mut options = labels;
            options.push("Done (No more labels)".to_string());
            let keyboard = make_keyboard(options.iter().map(|s| s.as_str()).collect(), 2);
            bot.send_message(msg.chat.id, "Select Labels (one by one):")
                .reply_markup(keyboard)
                .await?;
            dialogue.update(State::ReceiveLabels { data }).await?;
        }
        Err(e) => {
            warn!(error = ?e, "Failed to fetch labels, skipping labels step");
            bot.send_message(msg.chat.id, "As a... (Role)")
                .reply_markup(KeyboardRemove::new())
                .await?;
            dialogue.update(State::ReceiveRole { data }).await?;
        }
    }
    Ok(())
}

#[instrument(skip(bot, dialogue, data))]
pub async fn receive_labels(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    let label = msg.text().unwrap_or_default().to_string();
    
    if label == "Done (No more labels)" {
        info!(labels = ?data.labels, "Finished selecting labels");
        bot.send_message(msg.chat.id, "As a... (Role)")
            .reply_markup(KeyboardRemove::new())
            .await?;
        dialogue.update(State::ReceiveRole { data }).await?;
    } else {
        info!(label = %label, "Added label");
        data.labels.push(label);
        bot.send_message(msg.chat.id, format!("Added {}. Select another or 'Done'.", data.labels.last().unwrap())).await?;
        dialogue.update(State::ReceiveLabels { data }).await?;
    }
    Ok(())
}

#[instrument(skip(bot, dialogue, data))]
pub async fn receive_role(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.role = msg.text().unwrap_or_default().to_string();
    info!(role = %data.role, "Received role");
    bot.send_message(msg.chat.id, "I need... (Function)").await?;
    dialogue.update(State::ReceiveFunction { data }).await?;
    Ok(())
}

#[instrument(skip(bot, dialogue, data))]
pub async fn receive_function(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.function = msg.text().unwrap_or_default().to_string();
    info!(function = %data.function, "Received function");
    bot.send_message(msg.chat.id, "So that... (Benefit)").await?;
    dialogue.update(State::ReceiveBenefit { data }).await?;
    Ok(())
}

#[instrument(skip(bot, dialogue, data))]
pub async fn receive_benefit(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.benefit = msg.text().unwrap_or_default().to_string();
    info!(benefit = %data.benefit, "Received benefit");
    bot.send_message(msg.chat.id, "Details and Assumptions:").await?;
    dialogue.update(State::ReceiveDetails { data }).await?;
    Ok(())
}

#[instrument(skip(bot, dialogue, data))]
pub async fn receive_details(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.details = msg.text().unwrap_or_default().to_string();
    info!("Received details");
    bot.send_message(msg.chat.id, "Acceptance Criteria (Gherkin style):").await?;
    dialogue.update(State::ReceiveAcceptanceCriteria { data }).await?;
    Ok(())
}

#[instrument(skip(bot, dialogue, data), fields(title = %data.title))]
pub async fn receive_acceptance_criteria(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.acceptance_criteria = msg.text().unwrap_or_default().to_string();
    info!("Received acceptance criteria, creating issue...");

    // Add Issue Type to labels
    let mut all_labels = data.labels.clone();
    all_labels.push(data.issue_type.clone());

    let body = format!(
        "**As a** {}\n**I need** {}\n**So that** {}\n\n### Details and Assumptions\n\n- {}\n\n### Acceptance Criteria\n\n```gherkin\n{}\n```",
        data.role, data.function, data.benefit, data.details, data.acceptance_criteria
    );

    bot.send_message(msg.chat.id, "Creating issue...").await?;

    match github::create_issue(&data.title, &body, all_labels).await {
        Ok(url) => {
            info!(url = %url, "Issue created successfully");
            bot.send_message(msg.chat.id, format!("Issue created and added to project: {}", url)).await?;
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to create issue");
            bot.send_message(msg.chat.id, format!("Failed to create issue: {}", e)).await?;
        }
    }

    dialogue.exit().await?;
    Ok(())
}

fn make_keyboard(options: Vec<&str>, width: usize) -> KeyboardMarkup {
    let mut keyboard = vec![];
    for chunk in options.chunks(width) {
        let row: Vec<KeyboardButton> = chunk.iter().map(|&s| KeyboardButton::new(s)).collect();
        keyboard.push(row);
    }
    KeyboardMarkup::new(keyboard).one_time_keyboard().resize_keyboard()
}

fn is_authorized(msg: &Message) -> bool {
    let chat_id = msg.chat.id.0;
    let thread_id = msg.thread_id.map(|id| id.0.0).unwrap_or(0);
    let user_id = msg.from.as_ref().map(|u| u.id.0).unwrap_or(0);

    info!(chat_id = %chat_id, thread_id = %thread_id, user_id = %user_id, "Checking authorization");

    // Check Chat ID
    let target_chat_id_str = std::env::var("TARGET_CHAT_ID").unwrap_or_default();
    if !target_chat_id_str.is_empty() {
        match target_chat_id_str.trim().parse::<i64>() {
            Ok(target_id) => {
                if target_id != chat_id {
                    warn!(target = %target_id, actual = %chat_id, "Chat ID mismatch");
                    return false;
                }
            }
            Err(e) => {
                warn!(target = %target_chat_id_str, error = ?e, "Failed to parse TARGET_CHAT_ID");
            }
        }
    }

    // Check Thread ID (Topic)
    let target_thread_id_str = std::env::var("TARGET_THREAD_ID").unwrap_or_default();
    if !target_thread_id_str.is_empty() {
        match target_thread_id_str.trim().parse::<i32>() {
            Ok(target_id) => {
                if target_id != thread_id {
                    warn!(target = %target_id, actual = %thread_id, "Thread ID mismatch");
                    return false;
                }
            }
            Err(e) => {
                warn!(target = %target_thread_id_str, error = ?e, "Failed to parse TARGET_THREAD_ID");
            }
        }
    }

    // Check User ID
    let allowed_ids_str = std::env::var("ALLOWED_USER_IDS").unwrap_or_default();
    if allowed_ids_str.is_empty() {
        info!("No ALLOWED_USER_IDS set, allowing all users in target chat/thread");
        return true;
    }

    let authorized = allowed_ids_str.split(',').any(|id| {
        match id.trim().parse::<u64>() {
            Ok(parsed_id) => parsed_id == user_id,
            Err(e) => {
                warn!(id = %id, error = ?e, "Failed to parse ID in ALLOWED_USER_IDS");
                false
            }
        }
    });

    if !authorized {
        warn!(user_id = %user_id, allowed = %allowed_ids_str, "User ID not in allowed list");
    } else {
        info!(user_id = %user_id, "User authorized");
    }

    authorized
}
