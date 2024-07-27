use chrono::Duration;
use serenity::all::{Context, Message, Timestamp, User, UserId};
use serenity::prelude::TypeMapKey;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn update_user_join_date(ctx: &Context, user: &User, join_date: i64) {
    if get_user_join_date(ctx, user).await.is_none() {
        let counter_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<UserJoinDate>()
                .expect("Expected UserJoinDate in TypeMap.")
                .clone()
        };
        {
            let mut counter = counter_lock.write().await;
            let _ = counter.entry(user.id).or_insert(join_date);
        }
    }
}

pub async fn get_user_join_date(ctx: &Context, user: &User) -> Option<i64> {
    let counter_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<UserJoinDate>()
            .expect("Expected UserJoinDate in TypeMap.")
            .clone()
    };
    let user_date_info = counter_lock.read().await;
    user_date_info.get(&user.id).copied()
}

pub struct UserJoinDate;

impl TypeMapKey for UserJoinDate {
    type Value = Arc<RwLock<HashMap<UserId, i64>>>;
}

pub struct UserContext;

impl TypeMapKey for UserContext {
    type Value = Arc<RwLock<HashMap<UserId, UserHistory>>>;
}

pub struct UserHistory {
    min: Timestamp,
    max: Timestamp,
    max_size: usize,
    history: VecDeque<(Timestamp, String)>,
}

impl Default for UserHistory {
    fn default() -> Self {
        UserHistory {
            min: Default::default(),
            max: Default::default(),
            max_size: 3,
            history: Default::default(),
        }
    }
}

impl UserHistory {
    fn recalculate_bounds(&mut self) {
        self.min = *self.history.iter().map(|(time, _)| time).min().unwrap();
        self.max = *self.history.iter().map(|(time, _)| time).max().unwrap();
    }

    pub fn push(&mut self, time: Timestamp, message: String) {
        match (time.cmp(&self.min), time.cmp(&self.max)) {
            (Ordering::Greater, Ordering::Less) => {
                if self.history.len() > self.max_size {
                    self.history.pop_front();
                }
                let index = self
                    .history
                    .iter()
                    .position(|(history_time, _)| time > *history_time)
                    .unwrap();
                self.history.insert(index, (time, message));
                self.recalculate_bounds();
            }
            (Ordering::Greater, Ordering::Greater) => {
                if self.history.len() > self.max_size {
                    self.history.pop_front();
                }
                self.history.push_back((time, message));
                self.recalculate_bounds();
            }
            _ => {}
        }
    }

    pub fn context(&self, time: Timestamp, context_window: Duration) -> Vec<String> {
        self.history
            .iter()
            .filter_map(|(history_timestamp, message)| {
                if (time.unix_timestamp() - history_timestamp.unix_timestamp())
                    < context_window.num_milliseconds()
                {
                    Some(message.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub async fn update_user_context(ctx: &Context, message: &Message) {
    let context_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<UserContext>()
            .expect("Expected UserContext in TypeMap.")
            .clone()
    };
    {
        let mut all_users_context = context_lock.write().await;
        let user_context = all_users_context
            .entry(message.author.id)
            .or_insert(UserHistory::default());
        user_context.push(message.timestamp, message.content.clone());
    }
}

pub async fn retrieve_user_context(ctx: &Context, message: &Message) -> Vec<String> {
    let context_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<UserContext>()
            .expect("Expected UserContext in TypeMap.")
            .clone()
    };
    {
        let all_users_context = context_lock.read().await;
        if let Some(user_context) = all_users_context.get(&message.author.id) {
            user_context.context(message.timestamp, Duration::minutes(5))
        } else {
            vec![]
        }
    }
}
