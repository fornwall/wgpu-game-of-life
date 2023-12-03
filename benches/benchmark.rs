use criterion::{criterion_group, criterion_main, Criterion};
use wgpu::{Device, Instance, Queue};

pub fn criterion_benchmark(c: &mut Criterion) {
    let benchmark_name = "Setup Device";
    c.bench_function(benchmark_name, |b| {
        b.iter(|| {
            pollster::block_on(setup_device());
        });
    });

    let (instance, device, queue) = pollster::block_on(setup_device());
    test_computer(&instance, &device, &queue, 1024, c);
    test_computer(&instance, &device, &queue, 2048, c);
}

fn test_computer(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cells_width: u32,
    criterion: &mut Criterion,
) {
    let cells_height = cells_width;

    let creator = wgpu_game_of_life::computer::ComputerFactory::new(device);
    let seed = 1;
    let initial_density = 50;
    let rule = &wgpu_game_of_life::rules::RULES[0];
    let mut computer = creator.create(
        device,
        cells_width,
        cells_height,
        rule,
        seed,
        initial_density,
        queue,
    );
    instance.poll_all(true);

    let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    queue.submit(std::iter::once(encoder.finish()));

    let benchmark_name = format!("width={cells_width}");
    criterion.bench_function(&benchmark_name, |b| {
        b.iter(|| {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            computer.enqueue(&mut encoder);
            queue.submit(std::iter::once(encoder.finish()));
            instance.poll_all(true);
        });
    });
}

async fn setup_device() -> (Instance, Device, Queue) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();
    (instance, device, queue)
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::new(1, 0))
        .nresamples(10_000)
        .measurement_time(std::time::Duration::new(3, 0));
    targets = criterion_benchmark
}

criterion_main!(benches);
