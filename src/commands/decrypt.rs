use base64;
use clap::Args as ClapArgs;

use fencryption_rust::crypto::Crypto;

#[derive(ClapArgs)]
/// Decrypts base64 encoded text (that has been encrypted) using the passed key
pub struct Command {
    /// Key used to decrypt
    #[clap(value_parser)]
    key: String,

    /// Data to decrypt
    #[clap(value_parser)]
    encrypted_data: String,
}

pub fn action(args: &Command) {
    let crypto = Crypto::new(args.key.as_bytes().to_vec());

    let decoded_enc_data =
        base64::decode(args.encrypted_data.as_bytes()).expect("Wrongly base64 encoded data");

    let dec_data = match crypto.decrypt(&decoded_enc_data) {
        Ok(dec) => dec,
        Err(e) => panic!("Failed to decrypt: {}", e),
    };

    println!("\nDecryption result:\n");
    println!("- byte array result: {:x?}", &dec_data);
    println!("- base 64 encoded result: {}", base64::encode(&dec_data));
    println!(
        "utf8 encoded result: {}",
        match std::str::from_utf8(&dec_data) {
            Ok(v) => v,
            Err(_) => "Invalid utf8 sequence",
        }
    );
}