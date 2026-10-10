# Rayap Musik
## Product Requirements Document (PRD)

**Versi:** 1.1 — Draft untuk persetujuan  
**Status:** Product scope ditetapkan; dukungan M4A bersifat wajib; asumsi teknis menunggu verifikasi  
**Platform:** Linux terminal  
**Bahasa implementasi:** Rust  
**Tipe aplikasi:** Local-first Terminal User Interface (TUI)

---

## 1. Ringkasan Produk

Rayap Musik adalah pemutar musik lokal berbasis terminal Linux. Aplikasi memungkinkan pengguna menjelajahi direktori musik, membaca metadata lagu, memutar audio, dan mengelola antrean playback menggunakan keyboard.

Rayap Musik mengutamakan fungsionalitas nyata, tampilan terminal yang rapi, serta perilaku yang konsisten. Aplikasi harus mampu memutar file audio yang didukung pada sistem Linux pengguna, tanpa memerlukan server musik atau koneksi internet saat digunakan.

### Prinsip desain

1. **Local-first:** seluruh musik dibaca langsung dari filesystem lokal.
2. **Keyboard-first:** fungsi utama dapat diakses menggunakan keyboard.
3. **Simple by design:** satu executable, arsitektur modular sederhana.
4. **Functional over decorative:** playback yang andal lebih penting daripada dekorasi UI.
5. **Predictable behavior:** sorting, queue, dan kontrol playback mengikuti aturan yang jelas.
6. **Safe failure:** kesalahan file, filesystem, atau audio ditangani tanpa crash yang tidak perlu.
7. **Future-friendly, not overengineered:** struktur kode boleh mendukung pengembangan berikutnya, tetapi fitur masa depan tidak dibangun sebelum diperlukan.

## 2. Target Pengguna dan Kebutuhan

### Target pengguna

Pengguna Linux yang memiliki koleksi musik lokal dan ingin memutarnya melalui terminal dengan antarmuka keyboard-driven yang informatif.

### Masalah yang diselesaikan

- Memutar koleksi musik tanpa membuka aplikasi desktop.
- Menavigasi koleksi melalui folder dan subfolder.
- Melihat judul, artis, album, durasi, dan progres playback.
- Memutar kumpulan lagu secara berurutan.
- Mengontrol playback dan volume tanpa mouse.

## 3. Tujuan dan Non-Goals

### Tujuan MVP

- Memutar audio lokal yang didukung.
- Menavigasi direktori secara manual dan memindai subfolder untuk membentuk queue.
- Membaca metadata dan mengurutkan lagu secara deterministik.
- Menyediakan playback control, volume control, dan queue navigation.
- Menampilkan status playback serta progress secara real-time.
- Menjaga UI tetap responsif selama audio dimainkan.
- Memulihkan kondisi terminal ketika aplikasi keluar.

### Di luar cakupan MVP

- Streaming dan layanan musik daring.
- Integrasi atau sinkronisasi Navidrome.
- Database dan indeks library persisten.
- Penyimpanan queue atau playlist antar-sesi.
- Album art, lirik, rekomendasi, dan pencarian berbasis kecerdasan buatan.
- Integrasi desktop media keys.
- Background daemon.
- Sistem konfigurasi persisten.
- Seek control.
- Playlist editor, penambahan lagu manual ke queue, dan penghapusan item queue.

## 4. Cara Menjalankan Aplikasi

Nama produk adalah **Rayap Musik**. Nama executable yang direncanakan adalah `rayap-musik`.

Contoh penggunaan:

`rayap-musik ~/Music`

Ketentuan:

- Pengguna memberikan direktori root melalui argumen CLI.
- Path relatif dan home-directory notation (`~`) perlu ditangani dengan benar. Jika ekspansi `~` tidak dilakukan shell atau tidak tersedia pada konteks pemanggilan, aplikasi harus memiliki perilaku yang jelas dan terdokumentasi.
- Path harus divalidasi sebelum UI dijalankan.
- Argumen tidak tersedia, path tidak ada, atau path bukan direktori menghasilkan pesan kesalahan yang jelas dan exit code non-zero.
- Aplikasi tidak melakukan scanning melalui jaringan atau mengakses layanan eksternal.
- Setelah aplikasi ditutup, playback dihentikan dan resource audio dilepaskan.

## 5. Functional Requirements

### FR-01 — Filesystem Browser

Pengguna dapat:

- Melihat direktori dan file audio dalam direktori aktif.
- Masuk ke subdirektori.
- Kembali ke direktori induk.
- Melihat pesan informatif ketika direktori kosong atau tidak memiliki file audio yang didukung.
- Melihat status error ketika direktori tidak dapat dibaca.

Browser menampilkan folder dan file audio yang dikenali oleh konfigurasi dukungan format. File non-audio tidak ditampilkan pada MVP.

Navigasi folder bersifat manual. Scanning rekursif digunakan untuk membentuk queue, bukan untuk menggantikan navigasi folder pada UI.

### FR-02 — Audio Metadata

Aplikasi berusaha membaca metadata berikut:

- Judul lagu.
- Nama artis.
- Nama album.
- Nomor trek.
- Durasi.

Metadata bersifat opsional. Ketika informasi tidak tersedia, aplikasi menggunakan fallback yang masuk akal:

- Judul: nama file tanpa ekstensi.
- Artis atau album: penanda bahwa informasi tidak tersedia.
- Durasi: penanda durasi tidak tersedia jika tidak dapat ditentukan.

Untuk folder kompilasi (multi-artis) di mana tag artis/album tidak tersedia atau berbeda antar-file, aplikasi menampilkan nilai per-track apa adanya dan tidak menormalkan atau menebak nilai gabungan.

Metadata yang tidak valid tidak boleh menyebabkan crash.

### FR-03 — Sorting

Pengurutan harus deterministik, tidak bergantung pada urutan filesystem dari OS.

Aturan:

1. Direktori ditampilkan sebelum file audio.
2. Direktori diurutkan secara alfabetis.
3. Lagu dengan nomor trek yang valid diurutkan berdasarkan nomor trek.
4. Lagu tanpa nomor trek ditempatkan setelah lagu bernomor dalam direktori yang sama.
5. Nama file digunakan sebagai tie-breaker.
6. Nomor trek dalam format seperti `1`, `01`, dan `1/12` harus ditangani secara konsisten.
7. Sorting tidak boleh berubah secara acak antara dua pemindaian terhadap isi direktori yang sama.

Pada scanning rekursif, urutan traversal antardirektori harus deterministik dan **dikunci** sebagai berikut: **depth-first, direktori sebelum file, dan di setiap level nama diurutkan alfabetis secara case-insensitive**. Queue mengikuti urutan traversal tersebut, dengan sorting lagu (nomor trek, lalu nama file) diterapkan di dalam setiap direktori.

### FR-04 — Audio Playback

Aplikasi menyediakan:

- Play.
- Pause dan resume.
- Next track.
- Previous track/restart.
- Auto-next ketika lagu selesai secara normal.
- Volume up/down.
- Tampilan status playback.
- Tampilan waktu berjalan, durasi total, dan progress jika informasi tersedia.

Restart hanya berarti memulai ulang lagu saat ini dari posisi nol. Aplikasi tidak menyediakan seek bebas ke posisi sembarang; tidak ada input atau elemen UI untuk seek.

Playback tidak boleh memblokir event loop UI. Implementasi harus menggunakan mekanisme audio yang memungkinkan input keyboard dan rendering UI tetap responsif.

### FR-05 — Queue Management

Queue berada di memori dan tidak disimpan antar-sesi.

Ketika pengguna memilih lagu dari browser:

1. Queue lama diganti seluruhnya.
2. Lagu terpilih menjadi item pertama.
3. Aplikasi memindai direktori lagu terpilih beserta seluruh subdirektorinya.
4. Lagu audio lainnya dalam subtree tersebut ditambahkan mengikuti urutan traversal dan sorting yang ditetapkan.
5. Lagu terpilih tidak boleh muncul dua kali.
6. Playback dimulai dari lagu terpilih.

Contoh:

Jika pengguna memilih `Music/Indie/Album A/03.flac`, queue dibentuk dari subtree `Music/Indie/Album A/`, dimulai dengan `03.flac`, lalu diikuti lagu lain dalam subtree berdasarkan urutan yang ditetapkan.

Queue tidak mencakup direktori di luar subtree tersebut. Jika direktori tidak memiliki lagu audio yang dapat ditemukan selain lagu terpilih, queue hanya berisi lagu itu.

Memilih item dalam panel queue langsung memindahkan playback ke item tersebut. Queue tidak berubah hanya karena pengguna menavigasi folder di browser.

### FR-06 — Queue Completion dan Playback Errors

- Ketika lagu selesai secara normal, aplikasi memainkan item berikutnya.
- Setelah item terakhir selesai, playback berhenti.
- Queue tidak mengulang otomatis.
- Ketika lagu gagal dibuka atau didekode, aplikasi menampilkan pesan error dan mencoba melewati lagu tersebut.
- Kegagalan satu lagu tidak boleh membuat aplikasi crash.
- Aplikasi harus membedakan akhir playback normal dari kegagalan playback.
- Jika seluruh item gagal diputar, aplikasi harus menghentikan percobaan dengan aman dan menampilkan status yang informatif.
- Tidak boleh ada loop tanpa akhir ketika semua file dalam queue gagal.

### FR-07 — Previous Track Behavior

Ketika tombol previous ditekan:

- Jika lagu sudah diputar lebih dari 3 detik, mulai ulang lagu saat ini.
- Jika lagu diputar selama 3 detik atau kurang, berpindah ke lagu sebelumnya jika tersedia.
- Jika tidak ada lagu sebelumnya, perilaku tetap aman dan konsisten.
- Penghitungan waktu harus berdasarkan posisi playback aktual, bukan durasi wall-clock yang tidak memperhitungkan pause.

### FR-08 — Volume

- Volume awal aplikasi adalah 70%.
- Setiap penekanan tombol volume mengubah nilai sebesar 5 poin persentase.
- Nilai volume dibatasi pada rentang 0–100%.
- Perubahan volume berlaku pada playback aplikasi.
- Volume kembali ke 70% ketika aplikasi dijalankan ulang karena konfigurasi persisten tidak termasuk MVP.

Volume aplikasi tidak menjamin tingkat kenyaringan fisik tertentu karena output audio dan pengaturan sistem dapat berbeda.

### FR-09 — User Interface

UI menggunakan gaya dark dashboard yang dirancang untuk terminal.

Panel utama:

1. **Library:** direktori aktif, navigasi, dan seleksi.
2. **Now Playing:** judul, artis, album, dan status playback.
3. **Progress:** waktu berjalan, durasi total, dan progress bar.
4. **Queue:** daftar antrean dan penanda lagu aktif.
5. **Status/Help:** pesan error, volume, dan shortcut utama.

Ketentuan:

- Fokus awal berada pada Library.
- Panel Library dan Queue dapat dipilih melalui Tab.
- Panel aktif harus terlihat jelas.
- UI tidak boleh mengontrol audio secara langsung; UI hanya menampilkan state dan meneruskan input.
- Tampilan harus tetap dapat digunakan pada terminal kecil.
- Gunakan layout ringkas ketika memungkinkan. Jika ukuran terminal terlalu kecil, tampilkan peringatan yang jelas daripada membiarkan UI berantakan atau panic.
- Warna dan kontras harus nyaman dibaca pada terminal dengan dukungan warna yang terbatas.

### FR-10 — Keyboard Mapping

| Tombol | Aksi |
|---|---|
| `↑` / `↓` | Pindah seleksi pada panel aktif |
| `Tab` | Beralih fokus Library/Queue |
| `Enter` | Masuk folder, memainkan lagu terpilih, atau memainkan item queue |
| `Backspace` | Kembali ke direktori induk saat Library aktif |
| `Space` | Play/Pause |
| `n` | Next track |
| `p` | Previous/restart sesuai aturan 3 detik |
| `+` / `=` | Volume naik |
| `-` | Volume turun |
| `q` | Keluar dan menghentikan playback |

Ketika Queue aktif, tombol Enter memainkan item yang dipilih. Ketika Library aktif, Enter masuk ke folder atau memainkan lagu yang dipilih.

Kontrol global seperti play/pause, next, previous, volume, dan quit tetap tersedia dari kedua panel.

### FR-11 — Symlink dan Filesystem Safety

- Symbolic link diabaikan pada MVP.
- Scanning rekursif tidak boleh mengikuti symlink direktori.
- Folder yang tidak dapat dibaca ditangani tanpa crash.
- File yang dihapus atau berubah selama sesi harus ditangani sebagai error yang dapat dipulihkan.
- Path file menjadi identitas track, bukan judul atau metadata.
- Pembacaan filesystem bersifat read-only; aplikasi tidak mengubah, memindahkan, atau menghapus koleksi musik.

### FR-12 — Lifecycle

Saat aplikasi dijalankan:

- Validasi argumen dan direktori root.
- Inisialisasi backend audio.
- Inisialisasi terminal dan UI.

Saat aplikasi keluar, baik secara normal maupun karena error yang dapat ditangani:

- Hentikan playback.
- Lepaskan resource audio.
- Pulihkan raw mode, alternate screen, dan visibility cursor.
- Kembalikan terminal ke kondisi normal.
- Laporkan error secara jelas tanpa menyembunyikannya.

## 6. Non-Functional Requirements

### NFR-01 — Platform dan kompatibilitas

Rayap Musik harus berjalan pada Linux dengan terminal interaktif dan backend audio yang kompatibel.

Dukungan distro dan backend spesifik harus diverifikasi selama investigasi teknis awal.

### NFR-02 — Responsivitas

- UI tetap menerima input ketika lagu sedang dimainkan.
- Pembacaan metadata dan scanning direktori tidak boleh membuat UI macet berkepanjangan.
- Tidak diperlukan asynchronous runtime tambahan kecuali terbukti dibutuhkan.

### NFR-03 — Stabilitas

- File rusak atau tidak didukung tidak menyebabkan crash yang tidak ditangani.
- Queue kosong, direktori kosong, dan direktori yang tidak dapat dibaca harus ditangani.
- Perilaku error dan transisi queue harus deterministik.

### NFR-04 — Maintainability

- Modular monolith dalam satu executable.
- Gunakan tipe data konkret pada tahap awal.
- Hindari abstraksi generik dan trait yang belum memiliki kebutuhan nyata.
- Hindari dependency tambahan tanpa alasan teknis.
- Kode harus diformat dan diuji.
- Jangan menonaktifkan test hanya agar build berhasil.

### NFR-05 — Future Compatibility

Konfigurasi persisten mungkin ditambahkan kemudian, tetapi tidak dibangun dalam MVP. Desain state dan modul tidak boleh sengaja membuat fitur tersebut sulit ditambahkan.

## 7. Kandidat Technology Stack

Stack awal yang direncanakan:

- **Rust:** bahasa utama.
- **Ratatui:** rendering TUI.
- **Crossterm:** input keyboard dan pengelolaan terminal.
- **Rodio:** audio playback.
- **Lofty:** pembacaan metadata audio.

Semua versi dan konfigurasi dependency harus diverifikasi sebelum implementasi.

### Dukungan format

**Format wajib:**

- **AAC/M4A (MPEG-4 Audio)** — wajib. Sebagian besar koleksi pengguna memakai format ini. MVP tidak dianggap selesai jika file M4A tidak benar-benar dapat diputar. Kegagalan mendukung M4A adalah **blocking**, bukan sekadar gap opsional.

**Format target:**

- MP3
- FLAC
- WAV
- OGG Vorbis
- Opus

Daftar target adalah target kompatibilitas, bukan klaim bahwa seluruh format otomatis tersedia melalui konfigurasi dependency apa pun. Agent wajib memverifikasi dukungan decoder, fitur crate, backend Linux, dan kebutuhan library sistem untuk setiap format, dengan prioritas verifikasi pada M4A.

Jika M4A tidak dapat didukung dengan stack terpilih (misalnya keterbatasan decoder AAC dalam MP4 pada Rodio/Symphonia), agent harus **berhenti, melaporkan gap secara eksplisit, dan menunggu persetujuan product owner** sebelum mengubah stack inti atau mengganti backend audio. Agent tidak boleh menurunkan status M4A menjadi opsional tanpa persetujuan.

## 8. Arsitektur dan Modul

```text
rayap-musik/
├── Cargo.toml
├── README.md
├── docs/
│   └── PRD.md
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── input.rs
│   ├── ui.rs
│   ├── library.rs
│   ├── player.rs
│   └── models.rs
└── tests/
    ├── library_tests.rs
    └── queue_tests.rs
```

### Tanggung jawab modul

- `main.rs`: lifecycle aplikasi, setup, loop utama, dan cleanup.
- `app.rs`: state aplikasi, seleksi, queue, status, serta pengambilan keputusan.
- `input.rs`: menerjemahkan input keyboard menjadi action.
- `ui.rs`: merender state aplikasi.
- `library.rs`: filesystem, deteksi format, metadata, sorting, dan scanning rekursif.
- `player.rs`: playback, pause/resume, volume, posisi, durasi, dan completion.
- `models.rs`: tipe data bersama seperti Track, LibraryItem, Queue, PlaybackState, Action, dan error.

Kontrak antarmodul:

- Library mengembalikan data terstruktur, bukan teks UI.
- Input menghasilkan action.
- App memvalidasi action dan memutuskan perubahan state serta perintah playback.
- Player mengelola audio dan melaporkan state/event yang relevan.
- UI hanya membaca state dan merender tampilan.

Struktur ini adalah panduan awal, bukan alasan untuk membuat abstraksi tambahan yang tidak dibutuhkan.

## 9. Acceptance Criteria

| ID | Kriteria penerimaan |
|---|---|
| AC-01 | Aplikasi dapat dijalankan dengan direktori root valid |
| AC-02 | Path hilang atau tidak valid menghasilkan pesan error dan exit code non-zero |
| AC-03 | Pengguna dapat masuk ke folder dan kembali ke direktori induk |
| AC-04 | Sorting folder dan lagu deterministik, termasuk fallback metadata |
| AC-05 | Memilih lagu mengganti queue dengan lagu terpilih di posisi pertama |
| AC-06 | Queue mencakup lagu audio dalam subtree direktori secara rekursif tanpa mengikuti symlink |
| AC-07 | Playback berjalan dan UI tetap responsif |
| AC-08 | Play/pause, next, previous/restart, dan volume bekerja sesuai spesifikasi |
| AC-09 | Progress dan waktu playback diperbarui ketika informasi tersedia |
| AC-10 | Metadata kosong menggunakan fallback yang konsisten |
| AC-11 | File rusak atau tidak didukung ditangani tanpa crash |
| AC-12 | Lagu yang gagal dilewati tanpa loop tanpa akhir |
| AC-13 | Queue berhenti setelah item terakhir dan tidak mengulang otomatis |
| AC-14 | Queue navigation memainkan item yang dipilih tanpa mengubah urutan queue |
| AC-15 | Direktori kosong dan direktori yang tidak terbaca ditangani dengan aman |
| AC-16 | UI tetap usable pada ukuran terminal yang memadai dan memberi peringatan ketika terlalu kecil |
| AC-17 | Terminal dipulihkan saat aplikasi keluar |
| AC-18 | Build, formatter, dan test relevan berhasil dijalankan |
| AC-19 | README memuat instruksi instalasi, dependency sistem, penggunaan, format yang didukung, shortcut, dan pengujian manual |
| AC-20 | Pengujian manual membuktikan file audio benar-benar terdengar melalui backend audio Linux yang didukung |
| AC-21 | File M4A/AAC nyata benar-benar dapat diputar dan terdengar melalui backend audio Linux, bukan hanya lolos kompilasi |

## 10. Test Strategy

### Unit tests

- Parsing dan sorting nomor trek.
- Sorting direktori dan file audio.
- Fallback metadata.
- Pembentukan queue rekursif.
- Queue dimulai dari lagu terpilih tanpa duplikasi.
- Previous/restart berdasarkan ambang 3 detik.
- Next dan akhir queue.
- Penanganan queue kosong.
- Batas volume 0–100%.

### Integration tests

- Scanning direktori bertingkat menggunakan fixture lokal.
- File tidak didukung dan metadata rusak.
- Direktori kosong dan direktori tidak terbaca jika dapat diuji secara aman.
- Penanganan kegagalan track dan transisi queue.
- Perilaku CLI untuk path valid dan invalid.

### Manual tests

- Memutar file audio nyata pada Linux.
- **Memutar file M4A nyata dan memastikan benar-benar terdengar (blocking untuk MVP).**
- Memastikan pause/resume dan pergantian lagu terdengar benar.
- Memastikan volume berubah.
- Memastikan UI tetap responsif.
- Memastikan terminal pulih setelah keluar normal dan setelah error yang dapat ditangani.

Test yang membutuhkan perangkat audio tidak boleh dinyatakan berhasil hanya karena proses kompilasi sukses.

## 11. Tahapan Implementasi

### Milestone 1 — Bootstrap dan UI Shell

- Buat crate dan struktur dasar.
- Verifikasi toolchain dan dependency TUI.
- Implementasikan terminal lifecycle, layout awal, panel focus, keyboard mapping dasar, dan quit.
- Belum ada playback atau scanning library.

**Exit criteria:** UI shell berjalan, keluar dengan aman, dan terminal pulih.

### Milestone 2 — Filesystem Browser

- Implementasikan validasi root, navigasi folder, deteksi file audio, metadata, sorting, dan error handling.
- Implementasikan scanning rekursif untuk kebutuhan queue.
- Tambahkan unit test library.

**Exit criteria:** browser dan urutan lagu konsisten pada fixture pengujian.

### Milestone 3 — Single-track Playback

- Integrasikan backend audio.
- Verifikasi lebih dulu bahwa decoder M4A benar-benar berfungsi sebelum melanjutkan implementasi playback.
- Implementasikan play/pause, volume, posisi, durasi, dan status playback.
- Verifikasi dukungan format lain dan kompatibilitas Linux.
- Pastikan UI tidak terblokir.

**Exit criteria:** minimal file MP3 dan M4A benar-benar dapat diputar dan kontrol dasar bekerja. M4A wajib lolos sebelum milestone dianggap selesai; jika M4A gagal, hentikan dan laporkan gap.

### Milestone 4 — Queue dan Transisi

- Implementasikan queue replacement rekursif.
- Implementasikan next, previous/restart, auto-next, dan stop di akhir.
- Implementasikan skip untuk track yang gagal.
- Tambahkan test queue dan error recovery.

**Exit criteria:** seluruh perilaku queue sesuai spesifikasi.

### Milestone 5 — Integration dan Hardening

- Integrasikan seluruh panel dan kontrol.
- Uji terminal kecil, direktori bermasalah, serta file audio yang gagal.
- Lengkapi README dan instruksi pengujian manual.
- Jalankan formatter, compiler checks, dan test suite.

**Exit criteria:** acceptance criteria terpenuhi dan hasil verifikasi dilaporkan.

## 12. Aturan Kerja untuk Coding Agent

1. Baca dokumen PRD secara menyeluruh sebelum mengubah kode.
2. Kerjakan satu milestone dalam satu waktu.
3. Sebelum coding, periksa environment, toolchain, dokumentasi dependency, dan risiko kompatibilitas yang relevan.
4. Ajukan rencana singkat beserta file yang akan diubah.
5. Jika ada keputusan arsitektur penting atau perubahan stack inti yang dibutuhkan, tunggu persetujuan product owner.
6. Jangan mengimplementasikan milestone berikutnya sebelum milestone aktif disetujui.
7. Jangan menambahkan database, jaringan, konfigurasi persisten, atau fitur di luar scope MVP.
8. Jangan mengklaim fitur atau test berfungsi jika belum benar-benar dijalankan.
9. Setelah setiap milestone, laporkan file yang berubah, keputusan teknis, hasil pemeriksaan, keterbatasan, dan risiko tersisa.
10. Jangan mengubah acceptance criteria atau menghapus test tanpa persetujuan.

## 13. Asumsi dan Hal yang Masih Terbuka

Hal berikut harus diverifikasi atau diputuskan sebelum bagian implementasi yang terkait dimulai.

### A. Verifikasi teknis

- **Decoder M4A/AAC yang benar-benar berfungsi (blocking untuk Milestone 3).** Harus diverifikasi dengan file M4A nyata sebelum `player.rs` ditulis serius.
- Versi Rust dan dependency yang kompatibel.
- Backend audio Linux dan library sistem yang dibutuhkan.
- Dukungan aktual setiap format target, dengan M4A sebagai prioritas utama.
- Cara mendeteksi akhir playback tanpa memblokir UI.
- Akurasi posisi dan durasi playback.
- Perilaku volume pada backend terpilih.
- Penanganan file audio yang gagal di tengah playback.
- Kompatibilitas terminal dan dukungan warna.

### B. Detail produk yang belum dikunci

- Apakah nama executable final tetap `rayap-musik`.
- Apakah volume dan warna terminal memerlukan opsi CLI di masa depan.
- Apakah file audio yang berubah atau dihapus ketika sesi berlangsung cukup ditangani melalui pesan error dan skip.

**Telah dikunci pada v1.1:** urutan traversal queue rekursif adalah depth-first, direktori sebelum file, alfabetis case-insensitive di setiap level (lihat FR-03). Restart hanya memulai ulang lagu dari posisi nol; seek bebas di luar scope (lihat FR-04). M4A adalah format wajib (lihat Section 7).

Keputusan yang belum diperlukan untuk MVP tidak boleh menjadi alasan menunda implementasi milestone yang tidak terpengaruh.

---

**Definisi selesai untuk MVP:** Rayap Musik dapat dijalankan dari terminal Linux, menjelajahi koleksi lokal, benar-benar memutar file audio yang didukung (dengan **M4A/AAC sebagai format wajib**), mengelola queue sesuai aturan, tetap responsif, menangani error secara aman, dan memulihkan terminal saat keluar. 
