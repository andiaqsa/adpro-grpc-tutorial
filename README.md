# adpro-grpc-tutorial

## Refleksi Modul 8: Rust gRPC

**1. Apa perbedaan utama antara metode unary, server streaming, dan bi-directional streaming, serta kapan sebaiknya masing-masing digunakan?**

**Perbedaan utama:**

* **Unary:** Mekanismenya mirip seperti REST API biasa, yaitu satu *request* dibalas dengan satu *response*. Cocok untuk ngambil data profil user, nyimpen form, atau transaksi tunggal (seperti `PaymentService` di tutorial ini).
* **Server Streaming:** Klien ngirim satu *request*, tapi server membalasnya dengan aliran data (*stream*) berkali-kali. Ini cocok buat ngirim data yang ukurannya gede biar gak bikin memori server jebol, atau buat fitur *live feed* seperti nampilin riwayat transaksi yang panjang.
* **Bi-directional Streaming:** Klien dan server bisa saling kirim pesan secara bersamaan lewat satu koneksi yang terus terbuka. Skenario paling cocok buat ini ya aplikasi yang butuh interaksi *real-time* tinggi, contohnya aplikasi *live chat* CS atau *game multiplayer*.

**2. Apa saja pertimbangan keamanan saat mengimplementasikan gRPC di Rust, terutama soal autentikasi, otorisasi, dan enkripsi data?**
Secara bawaan, gRPC jalan di atas HTTP/2 yang sangat mendukung enkripsi TLS (Transport Layer Security). Buat komunikasi antar-layanan (*microservices*), kita sebaiknya pakai mTLS (Mutual TLS) biar klien dan server sama-sama ngeverifikasi identitas satu sama lain. Selain itu, untuk autentikasi dan otorisasi, kita bisa ngecek token (seperti JWT) yang disisipkan di *metadata* (kalau di HTTP ini mirip *headers*) dengan memanfaatkan *interceptor* / *middleware* di library Tonic.

**3. Tantangan apa yang mungkin muncul saat menangani bi-directional streaming di gRPC Rust, khususnya untuk aplikasi chat?**
Tantangan utamanya ada di manajemen *concurrency* dan *state*. Di Rust, kalau kita mau *share* data antar banyak koneksi (misal nge- *broadcast* pesan ke satu *room* chat), kita harus hati-hati banget ngatur *lock* (pakai `Mutex` atau `RwLock` bersama `Arc`). Kalau salah penanganan, bisa terjadi *deadlock*. Selain itu, kita juga harus siap nanganin kondisi *error* seperti klien yang tiba-tiba putus koneksi, biar *background task* di server bisa berhenti dengan benar dan gak nyebabin *memory leak*.

**4. Apa kelebihan dan kekurangan menggunakan `tokio_stream::wrappers::ReceiverStream` untuk streaming response di Rust gRPC?**
* **Kelebihan:** Sangat praktis buat ngejembatanin *channel* asinkron bawaan Tokio (`mpsc`) dengan tipe *stream* yang diminta oleh Tonic. Kita jadi gampang banget nge-*spawn* *background task* yang bertugas ngumpulin data, lalu mengirimkannya lewat *channel* tersebut.
* **Kekurangan:** Ada sedikit *overhead* performa karena proses sinkronisasi antrean pesan di *channel*. Selain itu, kita harus jeli nentuin ukuran *buffer* dari *channel*-nya; kalau kekecilan bisa bikin *bottleneck*, kalau kebesaran malah boros memori RAM.

**5. Bagaimana cara menstrukturkan kode Rust gRPC agar lebih modular, mudah di-maintain, dan bisa dipakai ulang?**
Daripada menumpuk semua logika di `main.rs` atau `grpc_server.rs`, kodenya harus dipisah-pisah (Separation of Concerns). File definisi hasil *generate* dari Protobuf ditaruh di modul terpisah, lalu kita buat layer khusus untuk logika bisnis (*service layer*), dan layer khusus untuk akses ke database (*repository layer*). Sangat disarankan juga untuk mendefinisikan logika menggunakan `trait` di Rust, biar kodenya gampang di-*mock* saat kita bikin *unit test*.

**6. Pada implementasi `MyPaymentService`, langkah tambahan apa yang diperlukan untuk menangani logika pemrosesan pembayaran yang lebih kompleks?**
Di tutorial tadi, fungsi pembayarannya cuma '*dummy*' yang langsung nge-return `success: true`. Di dunia nyata, kita harus nambahin:
* Validasi input (misal ngecek apakah `amount` tidak minus).
* Cek saldo dan interaksi dengan database untuk nyimpen rekam jejak transaksi.
* Implementasi *idempotency key* biar kalau terjadi *timeout* jaringan dan klien nge-*retry request*, saldonya gak kepotong dua kali.
* Integrasi dengan *third-party payment gateway* (kayak Midtrans) beserta penanganan *error*-nya.

**7. Apa dampak penggunaan gRPC sebagai protokol komunikasi terhadap arsitektur sistem terdistribusi, terutama soal interoperabilitas dengan teknologi lain?**
gRPC bikin arsitektur *microservices* jadi sangat teratur karena semuanya dipaksa patuh pada "kontrak" API yang didefinisikan secara tegas di file `.proto`. Keuntungannya, tim yang pakai bahasa pemrograman beda (misal Rust, Go, dan Java) bisa saling komunikasi dengan sangat mulus karena kodenya di-*generate* otomatis. Kekurangan utamanya, gRPC kurang ramah kalau dipanggil langsung dari *browser* web (karena browser gak kasih kontrol penuh ke frame HTTP/2), jadi biasanya frontend butuh perantara seperti gRPC-Web atau API Gateway.

**8. Apa keuntungan dan kerugian HTTP/2 (protokol dasar gRPC) dibandingkan HTTP/1.1 atau WebSocket untuk REST API?**
* **Keuntungan:** HTTP/2 mendukung *multiplexing* (bisa ngirim banyak *request/response* berbarengan di satu koneksi TCP tanpa harus antre), pakai format *binary framing* yang lebih cepat diproses mesin, dan ada kompresi *header* (HPACK). Jauh lebih efisien dibanding HTTP/1.1.
* **Kerugian:** Karena formatnya udah biner, kodenya gak bisa langsung dibaca manusia. Ini bikin proses *debugging* manual jadi lebih ribet (gak bisa pakai *tools* sederhana kayak cURL biasa tanpa ekstensi khusus).

**9. Bagaimana perbandingan model request-response REST API dengan bi-directional streaming gRPC dalam hal komunikasi real-time?**
Model REST API itu kaku: klien harus nanya dulu baru server jawab. Kalau butuh fitur *real-time*, pengembang REST biasanya harus ngakalin pakai teknik *polling* yang bikin koneksi boros *resource* dan nambahin *latency*. Sebaliknya, gRPC *bi-directional streaming* ngebuka satu koneksi HTTP/2 yang persisten, jadi klien maupun server bisa saling *push* data secara instan kapan saja. Ini bikin responsivitas aplikasi jauh lebih ngebut untuk komunikasi *real-time*.

**10. Apa implikasi pendekatan berbasis skema (Protocol Buffers) gRPC dibandingkan dengan payload JSON di REST API yang lebih fleksibel?**
* **Protocol Buffers (gRPC):** Menggunakan skema yang ketat dan diubah menjadi format biner yang sangat padat. Implikasinya, ukuran data yang ditransfer super kecil dan proses *parsing*-nya secepat kilat. Sistem tipe datanya juga mencegah banyak *bug* saat *runtime*.
* **JSON (REST):** Sangat fleksibel (*schema-less*) dan gampang dibaca oleh manusia. Tapi implikasinya, ukuran datanya jauh lebih bengkak (karena ngirim teks *key* dan tanda baca berulang-ulang), proses serialisasinya makan waktu lebih lama, dan rentan terhadap *error* kalau tipe datanya tiba-tiba berubah tanpa sepengetahuan *client*.