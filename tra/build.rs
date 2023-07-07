fn main() {
    protobuf_codegen::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["protos/file_sync.proto"])
        .include("protos")
        .run()
        .expect("protoc");
}
