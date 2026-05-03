mod bot;
mod github;

use crate::bot::{
    cancel, help, new_issue, receive_acceptance_criteria, receive_benefit, receive_details,
    receive_function, receive_issue_type, receive_labels, receive_role, receive_title, start,
    Command, State,
};
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    prelude::*,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ogonek_gh=info,teloxide=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting GitHub Issue Bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    use crate::bot::is_authorized;
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::NewIssue].endpoint(new_issue))
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .filter(|msg: Message| is_authorized(&msg))
        .branch(command_handler)
        .branch(
            dptree::filter(|msg: Message| {
                msg.text()
                    .map(|t| t.to_lowercase() == "cancel")
                    .unwrap_or(false)
            })
            .endpoint(cancel),
        )
        .branch(case![State::Start].endpoint(start))
        .branch(case![State::ReceiveTitle].endpoint(receive_title))
        .branch(case![State::ReceiveIssueType { data }].endpoint(receive_issue_type))
        .branch(case![State::ReceiveLabels { data }].endpoint(receive_labels))
        .branch(case![State::ReceiveRole { data }].endpoint(receive_role))
        .branch(case![State::ReceiveFunction { data }].endpoint(receive_function))
        .branch(case![State::ReceiveBenefit { data }].endpoint(receive_benefit))
        .branch(case![State::ReceiveDetails { data }].endpoint(receive_details))
        .branch(
            case![State::ReceiveAcceptanceCriteria { data }].endpoint(receive_acceptance_criteria),
        );

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}
