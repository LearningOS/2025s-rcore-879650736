## 1.进程管理
- **需要回收的资源有哪些？**

  1. **用户态资源**：包括各线程的TID、用户栈（ustack）、Trap上下文（trap context）等，这些资源通过`TaskUserRes`管理，在进程退出时会被显式回收。
  
  2. **用户空间内存**：进程的代码段、数据段等用户空间内存页（通过`memory_set.recycle_data_pages()`回收）。
  
  3. **文件描述符**：进程打开的文件和其他I/O资源（通过`fd_table.clear()`关闭并释放）。
  
  4. **内核栈（kstack）**：每个线程的内核栈资源，当`TaskControlBlock`的引用计数归零时自动回收。
  
  5. **进程元数据**：如进程的`ProcessControlBlock`自身的内存（通过`remove_from_pid2process`解除全局映射）。

---

- **其他线程的 TaskControlBlock 可能被引用的位置及回收必要性**

  1. **进程的 `tasks` 数组**  
     - **位置**：进程的`ProcessControlBlock`通过`tasks`字段持有所有线程的`Arc<TaskControlBlock>`。  
     - **回收必要性**：必须回收。当主线程退出时，`process_inner.tasks.clear()`会移除所有`Arc`引用，若这些引用是唯一持有者，`TaskControlBlock`会被自动回收。

  2. **调度队列（如就绪队列/等待队列）**  
     - **位置**：若线程处于就绪状态（在`TaskManager`中）或等待定时器到期，其`TaskControlBlock`会被调度器引用。  
     - **回收必要性**：必须回收。`remove_inactive_task`会强制将这些线程移出调度队列，解除引用。

  3. **进程内部的同步原语（如 Mutex/Semaphore）**  
     - **位置**：若线程因等待锁或信号量阻塞，其`TaskControlBlock`会被同步原语的等待队列引用。  
     - **回收必要性**：必须回收。由于同步原语的生命周期限于进程内部，进程退出时这些阻塞线程会被直接移除（无需手动处理）。

## 2.mutex
两种 Mutex 实现的关键区别及其潜在问题如下：

---

### **1. `lock` 方法的结构差异**
- **Mutex1**：使用 `loop` 循环，确保任务被唤醒后**重新检查锁状态**。
  ```rust
  loop {
      let mut mutex_inner = self.inner.exclusive_access();
      if mutex_inner.locked {
          // 加入等待队列并阻塞
      } else {
          mutex_inner.locked = true; // 成功获取锁
          break;
      }
  }
  ```
- **Mutex2**：无循环，任务被唤醒后**直接退出 `lock` 函数**，不再检查锁状态。
  ```rust
  let mut mutex_inner = self.inner.exclusive_access();
  if mutex_inner.locked {
      // 加入等待队列并阻塞
  } else {
      mutex_inner.locked = true; // 成功获取锁
  }
  ```

**潜在问题**：  
Mutex2 中，当任务被唤醒后，若锁已被其他任务抢占，它会直接进入临界区（未持有锁），导致**数据竞争**。而 Mutex1 的循环设计保证了任务必须重新通过锁状态检查才能继续执行。

---

### **2. `unlock` 方法的行为差异**
- **Mutex1**：无论等待队列是否为空，**总是释放锁**（`locked = false`），再唤醒一个等待任务。
  ```rust
  mutex_inner.locked = false; // 强制释放锁
  if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
      add_task(waking_task); // 唤醒等待任务
  }
  ```
- **Mutex2**：仅在等待队列为空时释放锁（`locked = false`），否则**保持锁为占用状态**。
  ```rust
  if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
      add_task(waking_task); // 唤醒等待任务，但不释放锁
  } else {
      mutex_inner.locked = false; // 队列空时才释放
  }
  ```

**潜在问题**：  
Mutex2 中，若存在等待任务，锁保持 `locked = true`，但此时锁的实际持有者已调用 `unlock`。这会导致：
1. **新任务无法获取锁**：新任务看到 `locked = true` 会错误加入等待队列。
2. **死锁风险**：被唤醒的任务在 `lock` 中检查到 `locked = true`，再次阻塞，形成无限循环。

---

### **总结问题场景**
1. **数据竞争（Mutex2）**  
   任务 A 释放锁并唤醒任务 B，但任务 C 抢先获取锁。任务 B 被唤醒后不检查锁状态，直接进入临界区，与任务 C 产生数据竞争。

2. **死锁（Mutex2）**  
   所有任务均进入等待队列，但锁状态保持 `locked = true`（因队列非空），无人能实际获取锁，系统永久挂起。

## 3. 总结
实现了一个基于银行家算法的死锁检测与资源管理模块 ProcessLocker，支持互斥锁（mutex）和信号量（sem）两种资源的动态分配与回收。核心功能包括：

资源初始化：为线程分配初始资源并维护可用资源池；
预分配检测：通过模拟资源请求和安全性检查（简化版银行家算法）避免死锁；
动态调整：支持资源的申请（alloc）、释放（dealloc）及线程完成状态标记；
安全性验证：在 detect1 方法中通过临时副本模拟资源分配，确保系统始终处于安全状态。

## 4.荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

- rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。