use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceUsage {
    pub pid: u32,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub process_count: usize,
}

static SYSTEM: once_cell::sync::Lazy<Mutex<System>> =
    once_cell::sync::Lazy::new(|| Mutex::new(System::new_all()));

pub fn initialize() {
    let mut system = SYSTEM.lock().unwrap();
    refresh_processes(&mut system);
}

fn refresh_processes(system: &mut System) {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
}

pub fn collect(service_ids: &[String]) -> HashMap<String, ServiceUsage> {
    let roots: Vec<(String, u32)> = service_ids
        .iter()
        .filter_map(|id| crate::modules::process::get_service_pid(id).map(|pid| (id.clone(), pid)))
        .collect();

    if roots.is_empty() {
        return HashMap::new();
    }

    let mut system = SYSTEM.lock().unwrap();
    refresh_processes(&mut system);

    let cpu_count = system.cpus().len().max(1) as f32;
    let mut children: HashMap<Pid, Vec<Pid>> = HashMap::new();
    for (pid, process) in system.processes() {
        if let Some(parent) = process.parent() {
            children.entry(parent).or_default().push(*pid);
        }
    }

    let mut usage = HashMap::new();
    for (service_id, root_pid) in roots {
        let root = Pid::from(root_pid as usize);
        if let Some(snapshot) = aggregate_tree(&system, &children, root, cpu_count) {
            usage.insert(
                service_id,
                ServiceUsage {
                    pid: root_pid,
                    ..snapshot
                },
            );
        }
    }

    usage
}

fn aggregate_tree(
    system: &System,
    children: &HashMap<Pid, Vec<Pid>>,
    root: Pid,
    cpu_count: f32,
) -> Option<ServiceUsage> {
    if !system.processes().contains_key(&root) {
        return None;
    }

    let mut stack = vec![root];
    let mut visited = HashSet::new();
    let mut cpu_total = 0.0_f32;
    let mut memory_total = 0_u64;
    let mut process_count = 0;

    while let Some(pid) = stack.pop() {
        if !visited.insert(pid) {
            continue;
        }

        if let Some(process) = system.process(pid) {
            cpu_total += process.cpu_usage();
            memory_total = memory_total.saturating_add(process.memory());
            process_count += 1;
        }

        if let Some(process_children) = children.get(&pid) {
            stack.extend(process_children.iter().copied());
        }
    }

    if process_count == 0 {
        return None;
    }

    Some(ServiceUsage {
        pid: root.as_u32(),
        cpu_percent: (cpu_total / cpu_count).clamp(0.0, 100.0),
        memory_bytes: memory_total,
        process_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::get_current_pid;

    fn current_system() -> (System, Pid) {
        let mut system = System::new_all();
        refresh_processes(&mut system);
        (system, get_current_pid().expect("current pid"))
    }

    #[test]
    fn aggregates_current_process_tree() {
        let (system, root) = current_system();
        let children = HashMap::new();
        let usage = aggregate_tree(&system, &children, root, 1.0).expect("current process usage");
        assert!(usage.memory_bytes > 0);
        assert_eq!(usage.process_count, 1);
        assert!(usage.cpu_percent >= 0.0);
    }

    #[test]
    fn returns_empty_for_unknown_services() {
        let usage = collect(&["does-not-exist".to_string()]);
        assert!(usage.is_empty());
    }
}
