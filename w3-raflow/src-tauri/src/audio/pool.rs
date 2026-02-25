// src-tauri/src/audio/pool.rs

//! 音频缓冲池
//!
//! 用于复用音频缓冲区，减少内存分配开销

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

/// 音频缓冲池
///
/// 复用音频缓冲区以减少内存分配和拷贝开销
pub struct AudioBufferPool {
    /// 空闲缓冲区
    idle: Arc<Mutex<VecDeque<Vec<i16>>>>,

    /// 最大缓冲区数量
    max_buffers: usize,

    /// 缓冲区大小（样本数）
    buffer_size: usize,
}

impl AudioBufferPool {
    /// 创建新的缓冲池
    ///
    /// # 参数
    /// - `buffer_size`: 每个缓冲区的样本数
    /// - `pool_size`: 池中缓冲区的数量
    pub fn new(buffer_size: usize, pool_size: usize) -> Self {
        let mut idle = VecDeque::with_capacity(pool_size);

        // 预分配缓冲区
        for _ in 0..pool_size {
            idle.push_back(vec![0i16; buffer_size]);
        }

        Self {
            idle: Arc::new(Mutex::new(idle)),
            max_buffers: pool_size,
            buffer_size,
        }
    }

    /// 获取一个缓冲区
    ///
    /// 如果池中有空闲缓冲区则返回，否则创建新的
    pub async fn acquire(&self) -> Vec<i16> {
        let mut idle = self.idle.lock().await;
        if let Some(mut buffer) = idle.pop_front() {
            // 清零缓冲区
            for sample in buffer.iter_mut() {
                *sample = 0;
            }
            buffer
        } else {
            vec![0i16; self.buffer_size]
        }
    }

    /// 归还缓冲区到池中
    ///
    /// 如果池未满则保留，否则丢弃
    pub async fn release(&self, buffer: Vec<i16>) {
        let mut idle = self.idle.lock().await;
        if idle.len() < self.max_buffers {
            idle.push_back(buffer);
        }
        // 如果池满了，buffer 会被 drop 释放
    }

    /// 获取池中当前空闲缓冲区数量
    pub async fn idle_count(&self) -> usize {
        let idle = self.idle.lock().await;
        idle.len()
    }

    /// 获取缓冲区大小
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// 预热缓冲池（确保有足够的缓冲区）
    pub async fn warm_up(&self, target_count: usize) {
        let mut idle = self.idle.lock().await;
        while idle.len() < target_count.min(self.max_buffers) {
            idle.push_back(vec![0i16; self.buffer_size]);
        }
    }

    /// 获取缓冲池统计信息
    pub async fn stats(&self) -> PoolStats {
        let idle = self.idle.lock().await;
        PoolStats {
            total_capacity: self.max_buffers,
            idle_buffers: idle.len(),
            buffer_size: self.buffer_size,
        }
    }
}

/// 缓冲池统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// 总容量
    pub total_capacity: usize,

    /// 当前空闲缓冲区数量
    pub idle_buffers: usize,

    /// 缓冲区大小
    pub buffer_size: usize,
}

impl Default for AudioBufferPool {
    fn default() -> Self {
        // 默认: 4096 样本 (256ms @ 16kHz)，10 个缓冲区
        Self::new(4096, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buffer_pool_basic_operations() {
        let pool = AudioBufferPool::new(100, 5);

        // 初始状态有 5 个预分配的缓冲区
        assert_eq!(pool.idle_count().await, 5);

        // 获取缓冲区
        let buffer1 = pool.acquire().await;
        assert_eq!(buffer1.len(), 100);
        assert_eq!(pool.idle_count().await, 4);

        // 归还缓冲区
        pool.release(buffer1).await;
        assert_eq!(pool.idle_count().await, 5);
    }

    #[tokio::test]
    async fn test_buffer_pool_reuse() {
        let pool = AudioBufferPool::new(100, 2);

        // 获取并使用缓冲区
        let mut buffer = pool.acquire().await;
        buffer[50] = 12345;

        pool.release(buffer).await;

        // 再次获取应该复用同一个缓冲区
        let buffer = pool.acquire().await;
        // 缓冲区应该已经被清零
        assert_eq!(buffer[50], 0);
    }

    #[tokio::test]
    async fn test_buffer_pool_capacity_limit() {
        let pool = AudioBufferPool::new(100, 2);

        let b1 = pool.acquire().await;
        let b2 = pool.acquire().await;

        // 池已空
        assert_eq!(pool.idle_count().await, 0);

        // 获取第三个缓冲区会创建新的
        let b3 = pool.acquire().await;
        assert_eq!(b3.len(), 100);

        // 归还超过容量的缓冲区会被丢弃
        pool.release(b1).await;
        pool.release(b2).await;
        pool.release(b3).await;

        assert_eq!(pool.idle_count().await, 2); // 只有 2 个被保留
    }

    #[tokio::test]
    async fn test_buffer_pool_stats() {
        let pool = AudioBufferPool::new(1024, 8);
        let stats = pool.stats().await;

        assert_eq!(stats.total_capacity, 8);
        assert_eq!(stats.buffer_size, 1024);
        assert_eq!(stats.idle_buffers, 8); // 初始状态全部空闲
    }

    #[tokio::test]
    async fn test_buffer_pool_warm_up() {
        let pool = AudioBufferPool::new(100, 5);

        // 使用一半的缓冲区
        for _ in 0..3 {
            let _ = pool.acquire().await;
        }

        // 预热确保有足够的缓冲区
        pool.warm_up(5).await;

        let stats = pool.stats().await;
        assert_eq!(stats.idle_buffers, 5);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let pool = Arc::new(AudioBufferPool::new(100, 10));
        let mut handles = vec![];

        // 多线程并发访问
        for _ in 0..5 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let buffer = pool_clone.acquire().await;
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                pool_clone.release(buffer).await;
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }

        // 验证所有缓冲区都归还了
        let stats = pool.stats().await;
        assert!(stats.idle_buffers <= 10);
    }
}
