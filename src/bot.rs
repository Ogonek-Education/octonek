use crate::github;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
    utils::command::BotCommands,
};

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub struct IssueData {
    pub title: String,
    pub role: String,
    pub function: String,
    pub benefit: String,
    pub details: String,
    pub acceptance_criteria: String,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveTitle,
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

pub async fn start(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    if !is_authorized(&msg) {
        bot.send_message(msg.chat.id, "Unauthorized.").await?;
        return Ok(());
    }
    bot.send_message(msg.chat.id, "Send /newissue to start creating a GitHub issue.").await?;
    dialogue.exit().await?;
    Ok(())
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

pub async fn cancel(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelled.").await?;
    dialogue.exit().await?;
    Ok(())
}

pub async fn new_issue(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    if !is_authorized(&msg) {
        bot.send_message(msg.chat.id, "Unauthorized.").await?;
        return Ok(());
    }
    bot.send_message(msg.chat.id, "What is the title of the issue?").await?;
    dialogue.update(State::ReceiveTitle).await?;
    Ok(())
}

pub async fn receive_title(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    let title = msg.text().unwrap_or_default().to_string();
    let data = IssueData {
        title,
        ..Default::default()
    };
    bot.send_message(msg.chat.id, "As a... (Role)").await?;
    dialogue.update(State::ReceiveRole { data }).await?;
    Ok(())
}

pub async fn receive_role(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.role = msg.text().unwrap_or_default().to_string();
    bot.send_message(msg.chat.id, "I need... (Function)").await?;
    dialogue.update(State::ReceiveFunction { data }).await?;
    Ok(())
}

pub async fn receive_function(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.function = msg.text().unwrap_or_default().to_string();
    bot.send_message(msg.chat.id, "So that... (Benefit)").await?;
    dialogue.update(State::ReceiveBenefit { data }).await?;
    Ok(())
}

pub async fn receive_benefit(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.benefit = msg.text().unwrap_or_default().to_string();
    bot.send_message(msg.chat.id, "Details and Assumptions:").await?;
    dialogue.update(State::ReceiveDetails { data }).await?;
    Ok(())
}

pub async fn receive_details(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.details = msg.text().unwrap_or_default().to_string();
    bot.send_message(msg.chat.id, "Acceptance Criteria (Gherkin style):").await?;
    dialogue.update(State::ReceiveAcceptanceCriteria { data }).await?;
    Ok(())
}

pub async fn receive_acceptance_criteria(bot: Bot, msg: Message, dialogue: MyDialogue, mut data: IssueData) -> HandlerResult {
    data.acceptance_criteria = msg.text().unwrap_or_default().to_string();

    let body = format!(
        "**As a** {}\n**I need** {}\n**So that** {}\n\n### Details and Assumptions\n\n- {}\n\n### Acceptance Criteria\n\n```gherkin\n{}\n```",
        data.role, data.function, data.benefit, data.details, data.acceptance_criteria
    );

    bot.send_message(msg.chat.id, "Creating issue...").await?;

    match github::create_issue(&data.title, &body).await {
        Ok(url) => {
            bot.send_message(msg.chat.id, format!("Issue created: {}", url)).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Failed to create issue: {}", e)).await?;
        }
    }

    dialogue.exit().await?;
    Ok(())
}

fn is_authorized(msg: &Message) -> bool {
    let allowed_ids_str = std::env::var("ALLOWED_USER_IDS").unwrap_or_default();
    if allowed_ids_str.is_empty() {
        return true; 
    }

    let user_id = msg.from.as_ref().map(|u| u.id.0).unwrap_or(0);
    allowed_ids_str
        .split(',')
        .any(|id| id.trim().parse::<u64>().map(|parsed_id| parsed_id == user_id).unwrap_or(false))
}
