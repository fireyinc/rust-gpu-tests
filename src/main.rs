use gpu_testing::run;

fn main() {
    pollster::block_on(run(250., 250.));
}
