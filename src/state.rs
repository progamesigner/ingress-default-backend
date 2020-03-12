use {
    crate::env::Env,
    prometheus::{
        proto::MetricFamily, CounterVec, HistogramOpts, HistogramTimer, HistogramVec, Opts,
        Registry,
    },
};

#[derive(Clone, Debug)]
pub struct State {}

#[derive(Clone, Debug)]
pub struct MetricState {
    registry: Registry,
}

#[derive(Clone, Debug)]
pub struct ServiceState {
    request_counter: CounterVec,
    request_duration: HistogramVec,
}

impl State {
    pub fn new() -> (ServiceState, MetricState) {
        let registry = Registry::new();

        let namespace = Env::parse_metric_namespace();
        let subsystem = Env::parse_metric_subsystem();

        let request_counter = CounterVec::new(
            Opts::new("request_count_total", "Counter of HTTP requests made.")
                .namespace(&namespace)
                .subsystem(&subsystem),
            &["proto"],
        )
        .unwrap();

        let request_duration = HistogramVec::new(
            HistogramOpts::new(
                "request_duration_milliseconds",
                "Histogram of the time (in milliseconds) each request took.",
            )
            .namespace(&namespace)
            .subsystem(&subsystem)
            .buckets(vec![
                0.001, 0.003, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
            &["proto"],
        )
        .unwrap();

        registry
            .register(Box::new(request_counter.clone()))
            .unwrap();
        registry
            .register(Box::new(request_duration.clone()))
            .unwrap();

        (
            ServiceState {
                request_counter,
                request_duration,
            },
            MetricState { registry },
        )
    }
}

impl MetricState {
    pub fn gather(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}

impl ServiceState {
    pub fn increase_request_counter(&self, proto: &str) {
        self.request_counter.with_label_values(&[proto]).inc()
    }

    pub fn start_timer(&self, proto: &str) -> HistogramTimer {
        self.request_duration
            .with_label_values(&[proto])
            .start_timer()
    }
}
