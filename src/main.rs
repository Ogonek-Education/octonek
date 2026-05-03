mod bot;
mod github;

use crate::bot::{
    cancel, help, new_issue, receive_acceptance_criteria, receive_benefit, receive_details,
    receive_function, receive_role, receive_title, start, Command, State,
};
use teloxide::{
    dispatching::{dialogue::{self, InMemStorage}, UpdateHandler},
    prelude::*,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    log::info!("Starting GitHub Issue Bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::NewIssue].endpoint(new_issue))
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Start].endpoint(start))
        .branch(case![State::ReceiveTitle].endpoint(receive_title))
        .branch(case![State::ReceiveRole { data }].endpoint(receive_role))
        .branch(case![State::ReceiveFunction { data }].endpoint(receive_function))
        .branch(case![State::ReceiveBenefit { data }].endpoint(receive_benefit))
        .branch(case![State::ReceiveDetails { data }].endpoint(receive_details))
        .branch(case![State::ReceiveAcceptanceCriteria { data }].endpoint(receive_acceptance_criteria));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}
