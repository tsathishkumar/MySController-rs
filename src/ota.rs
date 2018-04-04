use ihex::record::Record;
use ihex::writer;

pub fn some_funtion() {
  let records = &[
    Record::Data { offset: 0x0010, value: vec![0x48,0x65,0x6C,0x6C,0x6F] },
    Record::EndOfFile
  ];

  let result = writer::create_object_file_representation(records);
  if result.is_ok() {
    println!("{}", result.unwrap());
  }
}