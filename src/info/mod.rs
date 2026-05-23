use cosmic::iced::{
    futures::{SinkExt, Stream},
    stream,
};
use std::{thread, time::Duration};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};
use tokio::sync::mpsc;

use crate::Message;

mod cpu;
pub use self::cpu::*;

mod process;
pub use self::process::*;

pub fn worker() -> impl Stream<Item = Message> {
    stream::channel(1, async |mut output| {
        let (tx, mut rx) = mpsc::channel(1);

        //TODO: configurable refresh times
        let processes_refresh = Duration::from_millis(3000);
        let graph_refresh = sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;

        // Gather CPU information
        {
            let tx = tx.clone();
            thread::spawn(move || {
                let mut sys = System::new();
                loop {
                    sys.refresh_cpu_usage();
                    let cpus = sys.cpus();
                    let mut cpu_items = Vec::with_capacity(cpus.len());
                    for cpu in cpus {
                        cpu_items.push(CpuItem::new(cpu));
                    }
                    match tx.blocking_send(Message::Cpus(cpu_items)) {
                        Ok(()) => {}
                        Err(_) => break,
                    }

                    thread::sleep(graph_refresh);
                }
            });
        }

        // Gather process information
        thread::spawn(move || {
            //TODO: refresh users periodically?
            let users = Users::new_with_refreshed_list();
            let mut sys = System::new();
            loop {
                sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::nothing()
                        .with_cpu()
                        .with_disk_usage()
                        .with_memory()
                        .with_user(UpdateKind::OnlyIfNotSet),
                );
                let processes = sys.processes();
                let mut process_items = Vec::with_capacity(processes.len());
                for (_pid, process) in processes.iter() {
                    // Do not show threads
                    if process.thread_kind().is_some() {
                        continue;
                    }
                    process_items.push(ProcessItem::new(process, &users, processes_refresh));
                }
                match tx.blocking_send(Message::Processes(process_items)) {
                    Ok(()) => {}
                    Err(_) => break,
                }

                thread::sleep(processes_refresh);
            }
        });

        while let Some(msg) = rx.recv().await {
            output.send(msg).await.unwrap();
        }
    })
}
