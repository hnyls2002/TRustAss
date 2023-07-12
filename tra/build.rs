fn main() {
    protobuf_codegen::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["protos/file_sync.proto"])
        .include("protos")
        .run()
        .expect("protoc");

    // build the tonic in "protos/simple_test.proto"
    tonic_build::configure()
        .out_dir("src/rpc")
        .compile(&["protos/simple_test.proto"], &["protos"])
        .unwrap();
}
