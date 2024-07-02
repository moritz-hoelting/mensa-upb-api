use std::{collections::HashMap, sync::Arc};

use chrono::{NaiveDate, Utc};
use futures::StreamExt;
use itertools::Itertools;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::{Canteen, Menu};

#[derive(Debug, Clone, Default)]
pub struct MenuCache {
    cache: Arc<RwLock<HashMap<(NaiveDate, Canteen), Menu>>>,
}

impl MenuCache {
    pub async fn get_combined(&self, canteens: &[Canteen], date: NaiveDate) -> Menu {
        futures::stream::iter(canteens)
            .then(|canteen| async move { self.get(*canteen, date).await })
            .filter_map(|c| async { c })
            .fold(Menu::default(), |a, b| async move { a.merged(b) })
            .await
    }

    #[instrument(skip(self))]
    pub async fn get(&self, canteen: Canteen, date: NaiveDate) -> Option<Menu> {
        let query = (date, canteen);
        let (is_in_cache, is_cache_too_large) = {
            let cache = self.cache.read().await;
            (cache.contains_key(&query), cache.len() > 100)
        };
        if is_cache_too_large {
            self.clean_outdated().await;
        }
        if is_in_cache {
            let cache = self.cache.read().await;
            Some(cache.get(&query)?.clone())
        } else {
            debug!("Not in cache, fetching from network");

            let menu = Menu::new(date, canteen).await.ok()?;

            self.cache.write().await.insert(query, menu.clone());

            Some(menu)
        }
    }

    pub async fn clean_outdated(&self) {
        let today = Utc::now().date_naive();
        let outdated_keys = self
            .cache
            .read()
            .await
            .keys()
            .map(|x| x.to_owned())
            .filter(|(date, _)| date < &today)
            .collect_vec();
        let mut cache = self.cache.write().await;
        for key in outdated_keys {
            cache.remove(&key);
        }
    }
}
