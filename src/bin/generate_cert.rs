use rcgen::{Certificate, CertificateParams, DistinguishedName, SanType};
use std::fs;

fn main() -> anyhow::Result<()> {
    // 设置证书参数
    let mut params = CertificateParams::default();
    
    // 设置主题备用名称 (SAN) - 修正部分
    params.subject_alt_names = vec![
        SanType::DnsName("localhost".to_string()),
        // 可以添加更多SAN类型
        // SanType::IpAddress("127.0.0.1".parse().unwrap()),
    ];
    
    // 设置可分辨名称
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::CommonName, "QUIC Chat Server");
    distinguished_name.push(rcgen::DnType::OrganizationName, "My Company");
    params.distinguished_name = distinguished_name;
    
    // 生成证书
    let cert = Certificate::from_params(params)?;
    
    // 保存证书和私钥
    fs::write("cert.der", cert.serialize_der()?)?;
    fs::write("key.der", cert.serialize_private_key_der())?;
    
    // 可选：生成PEM格式
    fs::write("cert.pem", cert.serialize_pem()?)?;
    fs::write("key.pem", cert.serialize_private_key_pem())?;
    
    println!("Generated certificate files:");
    println!("- cert.der (DER format)");
    println!("- key.der (DER format)");
    println!("- cert.pem (PEM format)");
    println!("- key.pem (PEM format)");
    
    Ok(())
}