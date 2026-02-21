use axum::extract::Request;
use axum::http::{
    HeaderMap,
    header::{ACCEPT_LANGUAGE, CONTENT_LANGUAGE, HeaderName, HeaderValue},
};
use axum::middleware::Next;
use axum::response::Response;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    English,
    Indonesian,
}

impl Locale {
    pub fn as_language_tag(self) -> &'static str {
        match self {
            Locale::English => "en",
            Locale::Indonesian => "id",
        }
    }
}

tokio::task_local! {
    static REQUEST_LOCALE: Locale;
}

pub fn current_locale() -> Locale {
    REQUEST_LOCALE
        .try_with(|locale| *locale)
        .unwrap_or(Locale::English)
}

pub fn localize_message(message: &str) -> String {
    match current_locale() {
        Locale::English => message.to_string(),
        Locale::Indonesian => localize_to_indonesian(message),
    }
}

pub fn subscription_limit_message(feature: &str, limit: i32, current: i64) -> String {
    match current_locale() {
        Locale::English => format!(
            "You have reached the maximum of {} {} on the free plan (current: {})",
            limit, feature, current
        ),
        Locale::Indonesian => format!(
            "Anda telah mencapai batas maksimum {} {} pada paket gratis (saat ini: {})",
            limit, feature, current
        ),
    }
}

pub fn premium_required_message(feature: &str) -> String {
    match current_locale() {
        Locale::English => format!("{} requires a premium subscription", feature),
        Locale::Indonesian => format!("{} memerlukan langganan premium", feature),
    }
}

pub async fn with_request_locale(request: Request, next: Next) -> Response {
    let locale = resolve_locale(request.headers());
    REQUEST_LOCALE
        .scope(locale, async move {
            let mut response = next.run(request).await;
            response.headers_mut().insert(
                CONTENT_LANGUAGE,
                HeaderValue::from_static(locale.as_language_tag()),
            );
            response
        })
        .await
}

fn resolve_locale(headers: &HeaderMap) -> Locale {
    let x_language = HeaderName::from_static("x-language");
    if let Some(value) = headers
        .get(&x_language)
        .and_then(|value| value.to_str().ok())
    {
        if let Some(locale) = parse_locale(value) {
            return locale;
        }
    }

    if let Some(value) = headers
        .get(ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
    {
        for token in value.split(',') {
            if let Some(locale) = parse_locale(token) {
                return locale;
            }
        }
    }

    Locale::English
}

fn parse_locale(value: &str) -> Option<Locale> {
    let token = value
        .trim()
        .split(';')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if token == "id" || token.starts_with("id-") {
        return Some(Locale::Indonesian);
    }
    if token == "en" || token.starts_with("en-") {
        return Some(Locale::English);
    }
    None
}

fn localize_to_indonesian(message: &str) -> String {
    let translated = match message {
        "Internal Server Error" => "Terjadi kesalahan pada server internal",
        "Pocket created" => "Dompet berhasil dibuat",
        "Pocket updated" => "Dompet berhasil diperbarui",
        "Pocket deleted" => "Dompet berhasil dihapus",
        "Transaction saved" => "Transaksi berhasil disimpan",
        "Transaction updated" => "Transaksi berhasil diperbarui",
        "Transaction deleted" => "Transaksi berhasil dihapus",
        "Transaction restored" => "Transaksi berhasil dipulihkan",
        "Transfer successful" => "Transfer berhasil",
        "User data deleted" => "Data pengguna berhasil dihapus",
        "Base currency updated" => "Mata uang dasar berhasil diperbarui",
        "Investment removed" => "Investasi berhasil dihapus",
        "Investment updated" => "Investasi berhasil diperbarui",
        "Registration successful" => "Registrasi berhasil",
        "Login successful" => "Login berhasil",
        "Token refreshed" => "Token berhasil diperbarui",
        "Pocket not found" => "Dompet tidak ditemukan",
        "Default pocket not found" => "Dompet default tidak ditemukan",
        "Cannot delete the default pocket" => "Dompet default tidak dapat dihapus",
        "Pocket name cannot be empty" => "Nama dompet tidak boleh kosong",
        "Goal not found" => "Tujuan tidak ditemukan",
        "Goal name cannot be empty" => "Nama tujuan tidak boleh kosong",
        "Target amount must be positive" => "Jumlah target harus lebih dari nol",
        "Transaction not found" => "Transaksi tidak ditemukan",
        "Amount must be positive" => "Jumlah harus lebih dari nol",
        "Transfer amount must be positive" => "Jumlah transfer harus lebih dari nol",
        "Cannot transfer to the same pocket" => "Tidak dapat mentransfer ke dompet yang sama",
        "Insufficient funds in source pocket" => "Saldo dompet sumber tidak mencukupi",
        "End date cannot be before start date" => "Tanggal akhir tidak boleh sebelum tanggal mulai",
        "Subscription not found" => "Langganan tidak ditemukan",
        "User not found" => "Pengguna tidak ditemukan",
        "Invalid basis" => "Basis tidak valid",
        "Billing month must be null for monthly subscriptions" => {
            "Bulan tagihan harus kosong untuk langganan bulanan"
        }
        "Billing day must be between 1 and 31" => "Tanggal tagihan harus antara 1 dan 31",
        "Billing month is required for annual subscriptions" => {
            "Bulan tagihan wajib diisi untuk langganan tahunan"
        }
        "Billing month must be between 1 and 12" => "Bulan tagihan harus antara 1 dan 12",
        "Missing or invalid token" => "Token tidak ada atau tidak valid",
        "Invalid token" => "Token tidak valid",
        "Invalid user ID in token" => "ID pengguna dalam token tidak valid",
        "Invalid credentials" => "Email atau kata sandi tidak valid",
        "Password login not enabled for this account" => {
            "Login dengan kata sandi tidak diaktifkan untuk akun ini"
        }
        "Unsupported OAuth provider" => "Penyedia OAuth tidak didukung",
        "OAuth account missing email" => "Akun OAuth tidak memiliki email",
        "OAuth email not verified" => "Email OAuth belum diverifikasi",
        "Invalid refresh token" => "Refresh token tidak valid",
        "Token revoked" => "Token sudah dicabut",
        "Token expired" => "Token sudah kedaluwarsa",
        "Security alert: Token reuse detected" => {
            "Peringatan keamanan: penggunaan ulang token terdeteksi"
        }
        "Invalid OAuth token header" => "Header token OAuth tidak valid",
        "Invalid OAuth token algorithm" => "Algoritma token OAuth tidak valid",
        "Missing OAuth token key id" => "Key ID token OAuth tidak ditemukan",
        "Unknown OAuth token key" => "Kunci token OAuth tidak dikenal",
        "Invalid OAuth token key type" => "Tipe kunci token OAuth tidak valid",
        "Invalid OAuth token key" => "Kunci token OAuth tidak valid",
        "Invalid OAuth token" => "Token OAuth tidak valid",
        "Invalid OAuth token issuer" => "Penerbit token OAuth tidak valid",
        "Invalid OAuth token audience" => "Audiens token OAuth tidak valid",
        "Failed to fetch Google JWKS" => "Gagal mengambil Google JWKS",
        "Failed to parse Google JWKS" => "Gagal mengurai Google JWKS",
        "GOOGLE_CLIENT_ID must be set" => "GOOGLE_CLIENT_ID harus diatur",
        "Token creation failed" => "Gagal membuat token",
        "Password hashing failed" => "Gagal melakukan hash kata sandi",
        "Invalid password hash in DB" => "Hash kata sandi di DB tidak valid",
        "Unable to generate username" => "Tidak dapat membuat username",
        "User with this email or username already exists" => {
            "Pengguna dengan email atau username tersebut sudah ada"
        }
        "Username already taken" => "Username sudah digunakan",
        "ITICK_API_KEY not configured" => "ITICK_API_KEY belum dikonfigurasi",
        "Failed to parse iTick quote price" => "Gagal mengurai harga kuotasi iTick",
        "Failed to parse iTick EOD close" => "Gagal mengurai harga penutupan EOD iTick",
        "Failed to parse price" => "Gagal mengurai harga",
        "Failed to parse CoinGecko price" => "Gagal mengurai harga CoinGecko",
        "Failed to parse exchange rate" => "Gagal mengurai nilai tukar",
        _ => return localize_dynamic_to_indonesian(message),
    };

    translated.to_string()
}

fn localize_dynamic_to_indonesian(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("Invalid currency code: ") {
        return format!("Kode mata uang tidak valid: {}", rest);
    }
    if let Some(name) = message
        .strip_prefix("Investment ")
        .and_then(|value| value.strip_suffix(" not found"))
    {
        return format!("Investasi {} tidak ditemukan", name);
    }
    if let Some(name) = message
        .strip_prefix("Category '")
        .and_then(|value| value.strip_suffix("' not found"))
    {
        return format!("Kategori '{}' tidak ditemukan", name);
    }
    if let Some(ticker) = message
        .strip_prefix("Asset '")
        .and_then(|value| value.strip_suffix("' not supported"))
    {
        return format!("Aset '{}' tidak didukung", ticker);
    }
    if let Some(ticker) = message.strip_suffix(" is already in your portfolio") {
        return format!("{} sudah ada di portofolio Anda", ticker);
    }
    if let Some(rest) = message
        .strip_prefix("Updated ")
        .and_then(|value| value.strip_suffix(" assets"))
    {
        return format!("Memperbarui {} aset", rest);
    }
    if let Some(rest) = message
        .strip_prefix("Added ")
        .and_then(|value| value.strip_suffix(" to portfolio"))
    {
        return format!("{} ditambahkan ke portofolio", rest);
    }
    if let Some(rest) = message.strip_prefix("Unknown price source: ") {
        return format!("Sumber harga tidak dikenal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("iTick API connection failed: ") {
        return format!("Koneksi API iTick gagal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("iTick quote API returned error ") {
        return format!("API kuotasi iTick mengembalikan error {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Failed to parse iTick quote response: ") {
        return format!("Gagal mengurai respons kuotasi iTick: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("iTick quote API returned code ") {
        return format!("API kuotasi iTick mengembalikan kode {}", rest);
    }
    if let Some(rest) = message.strip_prefix("No quote data found for ") {
        return format!("Data kuotasi tidak ditemukan untuk {}", rest);
    }
    if let Some(rest) = message.strip_prefix("iTick EOD API connection failed: ") {
        return format!("Koneksi API EOD iTick gagal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("iTick EOD API returned error ") {
        return format!("API EOD iTick mengembalikan error {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Failed to parse iTick EOD response: ") {
        return format!("Gagal mengurai respons EOD iTick: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("iTick EOD fallback unauthorized: ") {
        return format!("Fallback EOD iTick tidak diizinkan: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("iTick EOD API returned code ") {
        return format!("API EOD iTick mengembalikan kode {}", rest);
    }
    if let Some(rest) = message.strip_prefix("No EOD data found for ") {
        return format!("Data EOD tidak ditemukan untuk {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Empty EOD data for ") {
        return format!("Data EOD kosong untuk {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Yahoo API connection failed: ") {
        return format!("Koneksi API Yahoo gagal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Yahoo API returned error ") {
        return format!("API Yahoo mengembalikan error {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Yahoo API returned explicit error: ") {
        return format!("API Yahoo mengembalikan error eksplisit: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Failed to parse Yahoo response: ") {
        return format!("Gagal mengurai respons Yahoo: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("No data found for ") {
        return format!("Data tidak ditemukan untuk {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Binance API connection failed: ") {
        return format!("Koneksi API Binance gagal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Binance API returned error for ") {
        return format!("API Binance mengembalikan error untuk {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Failed to parse Binance response: ") {
        return format!("Gagal mengurai respons Binance: {}", rest);
    }
    if let Some(rest) = message
        .strip_prefix("Failed to parse Binance price '")
        .and_then(|value| value.strip_suffix('\''))
    {
        return format!("Gagal mengurai harga Binance '{}'", rest);
    }
    if let Some(rest) = message.strip_prefix("CoinGecko API connection failed: ") {
        return format!("Koneksi API CoinGecko gagal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("CoinGecko API returned error: ") {
        return format!("API CoinGecko mengembalikan error: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Failed to parse CoinGecko response: ") {
        return format!("Gagal mengurai respons CoinGecko: {}", rest);
    }
    if let Some(rest) = message
        .strip_prefix("CoinGecko: No price found for ID '")
        .and_then(|value| value.strip_suffix('\''))
    {
        return format!("CoinGecko: Harga tidak ditemukan untuk ID '{}'", rest);
    }
    if let Some(rest) = message.strip_prefix("CoinGecko icon API connection failed: ") {
        return format!("Koneksi API ikon CoinGecko gagal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Frankfurter API connection failed: ") {
        return format!("Koneksi API Frankfurter gagal: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Frankfurter API returned error: ") {
        return format!("API Frankfurter mengembalikan error: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("Failed to parse Frankfurter response: ") {
        return format!("Gagal mengurai respons Frankfurter: {}", rest);
    }
    if let Some(rest) = message.strip_prefix("No rate found for ") {
        return format!("Nilai tukar tidak ditemukan untuk {}", rest);
    }

    message.to_string()
}

#[cfg(test)]
mod tests {
    use super::{Locale, localize_message, parse_locale};

    #[test]
    fn parse_locale_handles_language_and_region() {
        assert_eq!(parse_locale("id"), Some(Locale::Indonesian));
        assert_eq!(parse_locale("id-ID;q=0.9"), Some(Locale::Indonesian));
        assert_eq!(parse_locale("en-US"), Some(Locale::English));
    }

    #[tokio::test]
    async fn localize_message_uses_request_locale_context() {
        super::REQUEST_LOCALE
            .scope(Locale::Indonesian, async {
                let translated = localize_message("Pocket not found");
                assert_eq!(translated, "Dompet tidak ditemukan");
            })
            .await;
    }
}
