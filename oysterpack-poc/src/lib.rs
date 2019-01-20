/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]

use futures::{
    channel::{mpsc, oneshot},
    executor::ThreadPool,
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
    task::{Spawn, SpawnError, SpawnExt},
};
use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};

pub struct Cache<K, V, Command> {
    channel: mpsc::Sender<Command>,
    _phantom_key: PhantomData<K>,
    _phantom_value: PhantomData<V>,
}

impl<K, V> Cache<K, V, CacheCommand<K, V>>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    pub async fn get(
        &mut self,
        key: K,
    ) -> Result<oneshot::Receiver<Option<Arc<V>>>, mpsc::SendError> {
        let (sender, receiver) = oneshot::channel();
        let command = CacheCommand::<K, V>::Get(key, sender);
        await!(self.channel.send(command))?;
        Ok(receiver)
    }

    pub async fn set(&mut self, key: K, value: V) -> Result<(), mpsc::SendError> {
        let command = CacheCommand::<K, V>::Set(key, value);
        await!(self.channel.send(command))?;
        Ok(())
    }

    pub async fn remove(&mut self, key: K) -> Result<(), mpsc::SendError> {
        let command = CacheCommand::<K, V>::Remove(key);
        await!(self.channel.send(command))?;
        Ok(())
    }
}

impl<K, V> Cache<K, V, CacheCommand<K, V>>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    fn new(channel_buffer: usize, executor: &mut ThreadPool) -> Result<Self, SpawnError> {
        let (sender, receiver) = mpsc::channel(channel_buffer);
        let task = async move {
            let mut cache = HashMap::<K, Arc<V>>::new();
            let mut receiver = receiver;
            while let Some(command) = await!(receiver.next()) {
                match command {
                    CacheCommand::Get(key, reply_chan) => {
                        let value = cache.get(&key).map(|value| Arc::clone(value));
                        let _result = reply_chan.send(value);
                    }
                    CacheCommand::Set(key, value) => {
                        cache.insert(key, Arc::new(value));
                    }
                    CacheCommand::Remove(key) => {
                        cache.remove(&key);
                    }
                }
            }
        };
        executor.spawn(task)?;
        Ok(Self {
            channel: sender,
            _phantom_key: PhantomData,
            _phantom_value: PhantomData,
        })
    }
}

pub enum CacheCommand<K, V> {
    Get(K, oneshot::Sender<Option<Arc<V>>>),
    Set(K, V),
    Remove(K),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_cache() {
        let mut executor = ThreadPool::new().unwrap();
        let mut cache =
            Cache::<String, String, CacheCommand<_, _>>::new(10, &mut executor).unwrap();

        let task = async {
            let fname = "fname".to_string();
            await!(cache.set(fname.clone(), "Alfio".to_string())).unwrap();
            println!("name was set");
            let chan = await!(cache.get(fname.clone())).unwrap();
            println!("submitted Get request");
            let name = await!(chan).unwrap();
            println!("name: {:?}", name);
            assert_eq!(*name.unwrap(), "Alfio".to_string());
            await!(cache.remove(fname.clone())).unwrap();
            let chan = await!(cache.get(fname.clone())).unwrap();
            assert!(await!(chan).unwrap().is_none())
        };
        executor.run(task);
    }
}
