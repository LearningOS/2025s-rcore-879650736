## 1.描述程序出错行为
rustsbi 0.2.0
```
//ch2b_bad_address.rs输出为
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.

//ch2b_bad_instructions.rs输出为
[kernel] IllegalInstruction in application, kernel killed it.

//ch2b_bad_register.rs输出为
[kernel] IllegalInstruction in application, kernel killed it.
```

## 2.trap.S
1.  刚进入 __restore 时，sp 指向内核栈顶部。
使用场景：
处理完中断、异常或系统调用后，恢复用户进程的上下文并返回;
内核调度器选择新任务时，直接调用 __restore 加载新任务的上下文。

2.  处理了sstatus,sepc,sscratch
保证了内核态与用户态的安全切换;
实现内核栈与用户栈的物理隔离;
保存了正确的上下文。

3.  x2 (sp) 是栈指针寄存器，其值在上下文切换过程中需要手动调整。
x4(tp) 的实际值由内核控制，用户态无法直接操作。

4.  sp 指向内核栈的栈顶。sscratch保存 用户栈的栈顶地址

5.  在 __restore 函数中，状态切换（特权级切换）发生在 sret 指令。
sret 的行为由 RISC-V 特权架构定义，其
首先恢复特权级，然后恢复程序计数器：
sret 会将 pc 设置为 sepc 的值（异常发生时的下一条指令地址），从而继续执行用户程序。
再更新 sstatus。

6.  指令 csrrw sp, sscratch, sp 的作用是 原子性地交换 sp（栈指针）和 sscratch（监管者模式临时寄存器）的值。
执行后 sp 指向内核栈的顶部，sscratch 保存用户栈指针。

7. 通过ecall 指令，syscall_exit中的ecall。

## 3. 总结
在本次作业中，我实现了基于 RISC-V 的   sys_trace 系统调用（ID 410），主要包含三个功能：

1. **内存读写**  
   - trace_request=0时，将 id 视为用户空间指针，读取该地址的 u8 值返回。  
   - trace_request=1 时，将 data 的低 8 位写入 id 指向的用户地址，返回 0。  
   通过 unsafe 直接操作裸指针实现，暂未添加安全检查。

2. **系统调用统计**  
   - trace_request=2 时，查询当前任务调用指定系统调用（id）的总次数（含本次调用）。  
   在任务控制块（TaskControlBlock）中维护 syscall_counts 数组，每次系统调用触发时，在 syscall 中更新对应计数，sys_trace 直接返回该值。

3. **错误处理**  
   非法 trace_request 返回 -1。  

## 4.荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

- rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。