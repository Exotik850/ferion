use ferion::*;
fn main() -> Result<(), &'static str> {
    // Create a sample RION object
    let mut obj = RionObject::new();
    obj.add_field("name", -1000i64);
    // obj.add_field("age", 30u64);
    // obj.add_field("is_student", true);
    // obj.add_field("time", Utc::now());
    // obj.add_field(
    //     "age",
    //     RionField {
    //         field_type: RionFieldType::Int64Positive,
    //         value: vec![30],
    //     },
    // );
    // obj.add_field(
    //     "is_student",
    //     RionField {
    //         field_type: RionFieldType::Boolean,
    //         value: vec![1],
    //     },
    // );

    // Encode the object
    println!("Object: {:?}", obj);
    let encoded = obj.encode();
    println!("Encoded: {:?}", encoded);

    let decoded = RionField::from_slice(&encoded).unwrap();
    println!("Decoded: {:?}", decoded);

    let decoded_obj: RionObject = decoded.try_into().unwrap();
    println!("Final Object: {:?}", obj);
    // Decode the object
    // let decoded = RionObject::decode(&encoded)?;
    // println!("Decoded: {:?}", decoded);

    // Verify that the decoded object matches the original
    assert_eq!(obj, decoded_obj);
    println!("Encoding and decoding successful!");

    Ok(())
}
