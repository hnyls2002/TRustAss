fn main() {
    // protobuf_codegen::Codegen::new()
    //     .out_dir("src/protos")
    //     .inputs(&["protos/file_sync.proto"])
    //     .include("protos")
    //     .run()
    //     .expect("protoc");

    tonic_build::configure()
        .out_dir("src/protos")
        .compile(&["protos/simple_test.proto"], &["protos"])
        .unwrap();

    tonic_build::configure()
        .out_dir("src/protos")
        .compile(&["protos/replica.proto"], &["protos"])
        .unwrap();
}
