use crossbeam_channel::{self, Receiver, Sender};
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());

    // implement parallel map!
    let (itx, irx): (Sender<(usize, T)>, Receiver<(usize, T)>) = crossbeam_channel::bounded(1024);
    let (otx, orx): (Sender<(usize, U)>, Receiver<(usize, U)>) = crossbeam_channel::bounded(1024);

    // 创建线程池
    let mut threads = vec![];
    for _ in 0..num_threads {
        let irx_clone = irx.clone();
        let otx_clone = otx.clone();
        threads.push(thread::spawn(move || {
            while let Ok((index, input)) = irx_clone.recv() {
                // 计算输入
                let s = f(input);
                // 发送输出
                let _ = otx_clone.send((index, s));
            }
        }));
    }

    // 输入计算
    // TODO: O(n) -> O(1)
    let mut index = 0;
    while let Some(input) = input_vec.pop() {
        let _ = itx.send((index, input));
        index = index + 1;
        output_vec.push(U::default());
    }
    drop(itx);

    // 等待所有线程结束
    for handle in threads {
        handle.join().expect("panic in some handle");
    }
    drop(otx);

    // 收集结果
    while let Ok((index, output)) = orx.recv() {
        // output_vec.push(output);
        output_vec[index] = output;
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
