# Rayap Musik — Product Requirements Document (PRD)

**Versi:** 0.2.0  
**Status:** Product requirements ditetapkan; rencana teknis menunggu review dan persetujuan  
**Baseline:** Rayap Musik v0.1.0, PRD v1.1, QA Report 2026-10-10  
**Platform:** Linux  
**Bahasa implementasi:** Rust  
**Antarmuka:** Terminal User Interface (TUI)  
**Orientasi:** Local-first, personal use

---

## 1. Ringkasan Produk

Rayap Musik adalah pemutar musik lokal berbasis terminal yang berkembang menjadi **local music library manager**. Aplikasi membantu pengguna mengindeks, menjelajahi, mencari, mengelompokkan, dan memutar koleksi musik lokal, sekaligus mengelola queue, playlist, metadata, dan artwork.

Rayap Musik mengutamakan fungsionalitas nyata, perilaku yang konsisten, antarmuka keyboard-first, dan arsitektur sederhana yang dapat dikembangkan tanpa harus menulis ulang seluruh aplikasi.

Versi 0.2 memperluas Rayap Musik v0.1.0 yang telah memiliki filesystem browser, metadata dasar, playback, queue awal, dan kontrol TUI. Implementasi harus melanjutkan aplikasi yang sudah ada, bukan membangunnya kembali dari nol.

### Prinsip desain

1. **Local-first:** library, indeks, playlist, dan preferensi tersimpan secara lokal.
2. **Keyboard-first:** seluruh fungsi utama dapat digunakan melalui keyboard.
3. **Read-only terhadap file musik:** aplikasi membaca metadata dan artwork, tetapi tidak mengubah tag atau konten file audio.
4. **Predictable behavior:** queue, shuffle, repeat, sorting, dan playlist mengikuti aturan yang eksplisit.
5. **Playback reliability first:** keandalan playback lebih penting daripada dekorasi UI.
6. **Simple by design:** gunakan arsitektur modular tanpa abstraksi yang belum diperlukan.
7. **Future-friendly:** logika domain dipisahkan dari TUI agar GUI mungkin ditambahkan di masa depan, tetapi GUI tidak dibangun sekarang.
8. **Safe failure:** file hilang, metadata rusak, dan filesystem error ditangani tanpa crash yang tidak perlu.
9. **Verify before changing architecture:** perubahan dependency atau arsitektur penting harus disertai alasan teknis dan persetujuan product owner.

## 2. Baseline dan Aturan Pengembangan

Rayap Musik v0.1.0 dilaporkan lulus 30 dari 30 automated tests. QA juga melaporkan keberhasilan pemutaran M4A/AAC dan WAV pada perangkat nyata.

Baseline teknis yang tercatat:

- Rust 1.85.1.
- Rodio 0.21.1.
- Symphonia 0.5.5.
- Lofty 0.21.1.
- Ratatui 0.28.1.
- Crossterm 0.28.1.
- Linux dengan backend audio ALSA; PipeWire tersedia pada environment QA.

Catatan masalah v0.1.0:

- Opus belum didukung oleh stack decoder yang digunakan.
- Pembacaan metadata masih berpotensi memblokir UI pada direktori besar.
- Perilaku Space setelah queue selesai pernah memutar ulang track terakhir.
- `.gitignore` belum tersedia pada saat QA.

Informasi tersebut adalah baseline historis, bukan instruksi untuk mempertahankan dependency tanpa evaluasi. Periksa repository dan source aktual sebelum mengambil keputusan.

### Aturan kerja coding agent

1. Baca PRD ini secara menyeluruh sebelum mengubah kode.
2. Periksa source code, struktur modul, `Cargo.toml`, tests, README, Git status, dan dokumentasi dependency yang relevan.
3. Jangan menulis ulang aplikasi dari nol tanpa persetujuan.
4. Kerjakan **satu milestone dalam satu waktu**.
5. Sebelum setiap milestone, sampaikan rencana, file/modul yang akan berubah, risiko, keputusan teknis, dan metode verifikasi.
6. Tunggu persetujuan product owner sebelum melanjutkan ke milestone berikutnya.
7. Jangan mengubah requirement produk atau acceptance criteria secara diam-diam.
8. Jangan menambahkan dependency baru tanpa alasan teknis yang jelas.
9. Jangan mengubah dependency inti atau arsitektur sebelum meminta persetujuan jika perubahan tersebut diperlukan untuk memenuhi requirement.
10. Jangan mengklaim test, build, atau playback berhasil kecuali benar-benar dijalankan.
11. Jangan menghapus atau melemahkan test hanya agar build berhasil.
12. Setelah setiap milestone, laporkan perubahan, test yang dijalankan, hasil aktual, keterbatasan, dan risiko tersisa.

## 3. Tujuan dan Non-Goals

### 3.1 Tujuan v0.2

- Menyimpan root library dan preferensi antarsesi.
- Mengindeks koleksi musik lokal menggunakan database lokal.
- Menyediakan manual rescan dan filesystem watcher untuk mendeteksi perubahan otomatis.
- Menampilkan Songs, Albums, Artists, dan Playlists.
- Menyediakan pencarian global.
- Mendukung Play Now, Play Next, Add to Queue, dan penghapusan track dari queue.
- Mendukung shuffle dan tiga mode repeat.
- Menyediakan playlist persisten, termasuk operasi dasar, reorder, duplicate, sorting, serta impor/ekspor M3U.
- Mendukung seluruh format audio wajib.
- Menampilkan artwork dengan fallback yang konsisten.
- Menyediakan Play Mode layar penuh.
- Mengimplementasikan identitas visual Rust & Walnut, maskot ASCII rayap, dan tema alternatif.
- Menjaga UI responsif saat playback, indexing, dan filesystem watcher bekerja.
- Memisahkan logika domain dari UI agar kemungkinan GUI masa depan tidak mengharuskan penulisan ulang logika inti.

### 3.2 Di luar cakupan v0.2

- Streaming internet.
- Integrasi atau sinkronisasi Navidrome.
- Dukungan server/client.
- GUI desktop.
- Web API atau background daemon.
- Sinkronisasi lintas perangkat.
- Mengedit metadata/tag pada file audio asli.
- Menghapus atau memindahkan file audio asli.
- Mengunduh artwork dari internet.
- Rekomendasi musik berbasis AI.
- Lirik dan fitur sosial.
- Multi-root library.
- Bookmark untuk banyak direktori; yang disimpan hanya root library.
- Sistem plugin internal yang kompleks.

## 4. Target Pengguna dan Alur Utama

Rayap Musik ditujukan untuk penggunaan pribadi pada Linux dengan koleksi musik lokal.

Alur utama:

1. Pengguna menjalankan Rayap Musik.
2. Aplikasi memulihkan root library dan preferensi.
3. Aplikasi membuka indeks lokal dan memproses perubahan filesystem yang relevan.
4. Pengguna menjelajahi Songs, Albums, Artists, atau Playlists.
5. Pengguna mencari lagu atau koleksi menggunakan global search.
6. Pengguna memilih Play Now, Play Next, atau Add to Queue.
7. Pengguna mengelola queue dan playlist.
8. Pengguna berpindah antara Library dan Play Mode.
9. Aplikasi menyimpan perubahan persisten dan memulihkan terminal ketika keluar.

**Startup tidak boleh memulai playback secara otomatis.**

Jika root library belum dikonfigurasi atau tidak lagi tersedia, tampilkan pesan yang jelas dan sediakan langkah untuk mengatur root baru. Jangan crash atau menghapus database hanya karena root sementara tidak dapat diakses.

## 5. Library dan Indexing

### FR-LIB-01 — Root Library

- v0.2 menggunakan satu root directory.
- Root directory disimpan secara persisten.
- Pengguna tidak perlu memasukkan path setiap kali aplikasi dijalankan.
- Pengguna dapat mengubah root melalui tindakan yang jelas di aplikasi.
- Pemindaian mencakup root dan seluruh subdirektori secara rekursif.
- Symbolic link diabaikan dan symlink direktori tidak diikuti, kecuali perubahan perilaku ini disetujui secara eksplisit.
- Path yang tidak valid, hilang, atau tidak dapat diakses ditangani dengan pesan informatif.

### FR-LIB-02 — Database Lokal

- Indeks library dan playlist disimpan dalam database lokal.
- Database tidak memerlukan server terpisah.
- Engine database belum ditentukan dan harus direkomendasikan melalui investigasi teknis sebelum implementasi.
- Pertimbangkan kemudahan deployment, integritas data, migrasi schema, performa, dan maintenance.
- Schema harus mampu merepresentasikan track, path, album, artist, playlist, urutan playlist, dan status ketersediaan.
- Gunakan schema versioning/migration bila diperlukan.
- Database menyimpan indeks dan informasi yang diperlukan aplikasi; file audio tetap menjadi sumber konten musik.
- Kegagalan database harus ditangani dengan jelas. Jangan menghapus atau menginisialisasi ulang data secara diam-diam.

### FR-LIB-03 — Manual Scan dan Filesystem Watcher

Sediakan dua mekanisme pembaruan library:

**Manual rescan**
- Pengguna dapat memicu pemindaian ulang secara eksplisit.
- Rescan memeriksa perubahan file dan memperbarui indeks.

**Automatic watcher**
- Perubahan filesystem diproses saat terdeteksi.
- File baru, perubahan file, rename, dan penghapusan harus ditangani.
- UI tidak perlu direstart agar perubahan tercermin.
- Perubahan boleh diproses melalui debounce atau batching internal untuk menghindari pekerjaan berulang akibat satu operasi filesystem.
- Optimisasi internal tidak boleh membuat pembaruan tertunda tanpa batas.
- Pekerjaan scan dan metadata tidak boleh memblokir event loop UI dalam waktu lama.
- Jika watcher gagal atau tidak tersedia, tampilkan error dan pertahankan fungsi manual rescan.

### FR-LIB-04 — File Tidak Tersedia

- File yang diketahui hilang dari disk ditandai `unavailable`.
- Saat rescan, entri indeks yang sudah tidak ada dapat dibersihkan.
- Track unavailable tidak boleh menyebabkan crash saat dipilih atau diputar.
- Referensi track yang menjadi unavailable dihapus otomatis dari playlist.
- Aplikasi tidak boleh menghapus file asli dari filesystem.

### FR-LIB-05 — Metadata dan Fallback

Metadata dibaca secara read-only. Field utama:

- Title.
- Artist.
- Album.
- Track number.
- Duration.
- Embedded album artwork, jika tersedia.

Aturan fallback:

- Judul lagu menggunakan nama file tanpa ekstensi.
- Album dan artist menggunakan metadata jika tersedia.
- Jika metadata album atau artist tidak tersedia, gunakan folder sebagai fallback.
- Field yang tetap tidak tersedia harus ditampilkan dengan placeholder yang jelas dan konsisten.
- Metadata rusak atau tidak valid tidak boleh menyebabkan crash.
- Aplikasi tidak pernah menulis metadata kembali ke file musik.

### FR-LIB-06 — Identitas Track dan Duplikat

- Setiap path file merupakan satu entri track.
- Dua file yang berisi audio identik tetapi berada pada path berbeda tetap menjadi dua entri berbeda.
- Jangan menggabungkan track berdasarkan hash konten, judul, atau nama file.
- Path merupakan identitas sumber file, bukan metadata tampilan.
- Jangan menghapus atau menyembunyikan entri duplikat secara otomatis.

### FR-LIB-07 — Tampilan dan Pengelompokan Library

Library menyediakan minimal empat bagian:

- **Songs:** seluruh track yang terindeks.
- **Albums:** koleksi track berdasarkan album.
- **Artists:** koleksi track berdasarkan artist.
- **Playlists:** playlist yang disimpan pengguna.

Pengelompokan:

- Gunakan metadata terlebih dahulu.
- Gunakan folder sebagai fallback jika metadata album atau artist tidak tersedia.
- Album dengan nama yang sama digabung menjadi satu entri album, meskipun artisnya berbeda.
- Label artist untuk album gabungan menggunakan artist dari track pertama berdasarkan urutan deterministik library.
- Jangan mengganti label tersebut secara otomatis menjadi `Various Artists`.
- Urutan anggota album harus deterministik agar label artist tidak berubah secara acak antar-rescan.
- Track dalam album diurutkan berdasarkan nomor track, dengan nama file sebagai fallback.
- Jika nomor track tidak tersedia atau tidak valid, gunakan nama file sebagai fallback.
- Sorting Songs, Albums, Artists, dan hasil search harus konsisten. Aturan sorting yang belum ditentukan harus diusulkan dalam rencana teknis sebelum implementasi.

### FR-LIB-08 — Global Search

- Search mencakup songs, artists, albums, dan playlists.
- Hasil pencarian harus menunjukkan jenis item.
- Pengguna dapat menjalankan aksi yang sesuai dengan jenis hasil.
- Search berjalan sepenuhnya secara lokal.
- Tidak ada query atau metadata yang dikirim ke layanan eksternal.
- Default yang disarankan adalah pencocokan substring case-insensitive.
- Aturan pencocokan final harus dicatat dalam dokumentasi dan test.

### FR-LIB-09 — Aksi dari Library

Sediakan tiga aksi utama:

**Play Now**
- Mengganti queue lama dengan track atau koleksi yang dipilih.
- Memulai playback dari awal koleksi terpilih.

**Play Next**
- Menyisipkan track atau koleksi setelah track aktif.
- Satu track berarti satu lagu.
- Album, artist, atau folder berarti seluruh koleksi dalam urutan deterministik.

**Add to Queue**
- Menambahkan track atau koleksi ke akhir queue.

Aksi berlaku sesuai jenis item:

- Track: satu lagu.
- Album: seluruh track album.
- Artist: seluruh track yang dikelompokkan pada artist tersebut.
- Folder: seluruh lagu di folder dan subfolder.
- Playlist: track sesuai urutan playlist.

Jika sumber tidak memiliki track yang dapat diputar, tampilkan pesan informatif. Jangan mengganti queue aktif secara tidak sengaja.

## 6. Playback dan Queue

### FR-PLAY-01 — Playback Control

Pertahankan dan integrasikan fungsi v0.1:

- Play.
- Pause dan resume.
- Next track.
- Previous/restart.
- Volume up/down.
- Progress dan durasi jika tersedia.
- Auto-next setelah lagu selesai.
- Penanganan track yang gagal diputar.

Playback harus tetap responsif saat pengguna menavigasi library atau queue.

### FR-PLAY-02 — Queue sebagai State Terpisah

Queue playback berbeda dari library dan playlist tersimpan.

Pengguna dapat:

- Memilih track queue untuk langsung memutarnya.
- Menambahkan satu track.
- Menambahkan koleksi.
- Menghapus satu track dari queue.
- Mengubah queue melalui aksi playback.

Perubahan queue tidak boleh mengubah playlist tersimpan, kecuali pengguna melakukan aksi playlist secara eksplisit.

Menavigasi library tidak boleh dengan sendirinya mengganti queue.

### FR-PLAY-03 — Play Now

- Play Now mengganti seluruh queue lama.
- Track atau koleksi yang dipilih menjadi sumber queue baru.
- Playback dimulai dari track pertama sesuai urutan sumber.
- Item queue lama tidak dipertahankan.
- Jika sumber kosong atau tidak memiliki track playable, rekomendasi awal adalah mempertahankan queue lama dan menampilkan pesan. Konfirmasikan perilaku ini dalam rencana teknis sebelum implementasi.

### FR-PLAY-04 — Play Next

- Jika sumbernya satu track, sisipkan satu track.
- Jika sumbernya album, artist, folder, atau koleksi, sisipkan seluruh koleksi.
- Koleksi disisipkan setelah track aktif.
- Urutan internal koleksi harus deterministik.
- Jika tidak ada track aktif, coding agent harus mengusulkan perilaku yang jelas sebelum implementasi; jangan memilih perilaku secara diam-diam.

### FR-PLAY-05 — Shuffle

Shuffle merupakan pengaturan playback yang dapat diaktifkan dan dimatikan.

Saat shuffle diaktifkan:
- Acak seluruh queue setiap kali shuffle diaktifkan.
- Pertahankan track aktif agar tidak terjadi pergantian lagu yang tidak diminta hanya karena urutan queue berubah.

Saat shuffle dimatikan:
- Pulihkan urutan asli queue.
- Jangan mengganti track yang sedang aktif.
- Jangan memulai ulang track aktif.
- Track aktif harus tetap menjadi track aktif setelah urutan dipulihkan.

Implementasi harus menyimpan informasi yang cukup untuk mempertahankan urutan asli dan urutan playback saat shuffle aktif.

Jika queue berubah saat shuffle aktif, strategi untuk menyisipkan atau menghapus item harus dijelaskan dalam rencana teknis dan diuji secara deterministik.

### FR-PLAY-06 — Repeat

Sediakan tiga mode:

1. **Off:** playback berhenti setelah track terakhir selesai.
2. **Repeat All:** setelah track terakhir, playback kembali ke awal queue sesuai urutan playback aktif.
3. **Repeat One:** ulangi track aktif setelah selesai.

Shuffle dan repeat merupakan dua pengaturan terpisah dan harus dapat digunakan bersamaan dengan perilaku yang konsisten.

### FR-PLAY-07 — Track Error dan Queue Completion

- Jika track gagal dibuka atau didekode, tampilkan error yang ringkas dan coba lanjut ke track berikutnya.
- Jangan mencoba track yang sama tanpa akhir.
- Jika seluruh track gagal, hentikan percobaan dan tampilkan status yang jelas.
- Track unavailable tidak boleh membuat UI atau queue crash.
- Ketika queue selesai, perilaku harus sesuai mode repeat.
- Dengan repeat Off, queue tidak boleh secara tidak sengaja mengulang track terakhir.
- File yang dihapus ketika sesi berjalan harus ditangani dengan aman.

## 7. Playlist

### FR-PL-01 — Penyimpanan Playlist

- Playlist disimpan dalam format internal di database lokal.
- Playlist bertahan antar-sesi.
- M3U digunakan sebagai format impor dan ekspor, bukan sebagai satu-satunya penyimpanan internal.
- Detail schema dan representasi urutan playlist harus dijelaskan dalam rencana teknis.

### FR-PL-02 — Operasi Playlist

Pengguna dapat:

- Membuat playlist.
- Mengganti nama playlist.
- Menghapus playlist.
- Membuka dan memutar playlist.
- Menambahkan track ke playlist.
- Menghapus track dari playlist.
- Mengubah urutan track secara manual.
- Menggandakan playlist.
- Mengurutkan track playlist berdasarkan metadata.
- Menambahkan seluruh isi album ke playlist.
- Menambahkan seluruh isi artist ke playlist.
- Menambahkan seluruh isi folder ke playlist.

Menghapus playlist tidak boleh menghapus file audio asli.

### FR-PL-03 — Reorder dan Sorting

- Urutan manual playlist dipertahankan antar-sesi.
- Sorting metadata merupakan aksi eksplisit.
- Setelah pengguna mengubah urutan secara manual, urutan baru disimpan.
- Sorting harus deterministik dan memiliki tie-breaker yang konsisten.

### FR-PL-04 — Duplicate Playlist

- Duplicate membuat playlist baru dengan isi dan urutan yang sama pada saat duplikasi.
- Playlist hasil duplikasi independen dari playlist asal.
- Perubahan playlist asal tidak mengubah playlist duplikat secara otomatis.
- Nama default playlist duplikat harus berbeda dan dapat diedit.

### FR-PL-05 — M3U Import/Export

- Pengguna dapat mengimpor playlist M3U.
- Pengguna dapat mengekspor playlist internal ke M3U.
- Urutan track harus dipertahankan sejauh didukung format.
- Path relatif dan absolut ditangani secara konsisten.
- Path yang tidak ditemukan dilaporkan dan tidak boleh menyebabkan crash.
- Pengguna mendapatkan ringkasan item yang berhasil diimpor, dilewati, atau unavailable.
- Encoding dan variasi M3U yang didukung harus dicatat dalam README setelah diverifikasi.
- Import/export tidak boleh mengubah atau menghapus file musik asli.

### FR-PL-06 — Track Hilang dari Playlist

- Track yang menjadi unavailable dihapus otomatis dari playlist.
- Penghapusan referensi hanya memengaruhi data playlist internal.
- File asli tidak diubah.
- Penanganan harus mencegah referensi rusak memicu error berulang.

## 8. Preferensi dan Lifecycle

### FR-PREF-01 — Preferensi Persisten

Simpan preferensi yang relevan antar-sesi, termasuk:

- Root library.
- Tema terpilih.
- Volume.
- Preferensi UI yang diperlukan.
- Preferensi playback yang diputuskan perlu dipertahankan.

Perilaku preferensi playback seperti shuffle dan repeat harus ditetapkan dalam rencana teknis dan didokumentasikan.

Indeks dan playlist harus tersimpan di database lokal. Preferensi dapat disimpan di database yang sama atau file konfigurasi lokal; pilihan ini adalah keputusan teknis terbuka.

Jangan memulihkan state dengan cara yang membuat aplikasi auto-play tanpa persetujuan pengguna.

### FR-PREF-02 — Startup

- Pulihkan root dan preferensi.
- Buka tampilan Library.
- Jangan auto-play.
- Jika root tidak tersedia, tampilkan pesan dan opsi untuk memperbaikinya.
- Jika database gagal dibuka, jelaskan masalahnya dan tawarkan strategi recovery yang aman.
- Jangan menghapus database atau playlist secara diam-diam.

### FR-PREF-03 — Shutdown

Saat aplikasi ditutup:

- Hentikan playback dengan aman.
- Simpan perubahan persisten yang diperlukan.
- Hentikan watcher dan background tasks.
- Lepaskan resource audio.
- Pulihkan raw mode, alternate screen, dan cursor terminal.
- Laporkan error yang relevan secara jelas.

## 9. UI, Play Mode, Artwork, dan Tema

### FR-UI-01 — Tampilan Utama

Tampilan awal adalah Library.

Sediakan navigasi untuk:
- Songs.
- Albums.
- Artists.
- Playlists.
- Global search.
- Queue dan kontrol playback.

UI harus ringkas, keyboard-first, dan tetap terbaca pada terminal kecil.

### FR-UI-02 — Play Mode

Sediakan layar Now Playing/Play Mode layar penuh.

Tampilan ini menampilkan:
- Artwork album.
- Judul lagu.
- Artist.
- Album.
- Progress dan durasi jika tersedia.
- Status playback.
- Kontrol atau petunjuk shortcut yang relevan.

Pengguna dapat berpindah ke dan dari Play Mode menggunakan shortcut yang terdokumentasi.

Playback harus terus berjalan saat tampilan berubah. Play Mode hanya mengubah representasi UI, bukan membuat playback state terpisah.

### FR-UI-03 — Artwork

Urutan sumber artwork:

1. Embedded artwork pada metadata track.
2. Gambar lokal di folder album.
3. Placeholder bertema Rayap Musik.

Ketentuan:

- Artwork tidak diunduh dari internet.
- Artwork yang tidak tersedia atau tidak dapat ditampilkan harus menghasilkan fallback, bukan crash.
- Nama file gambar folder dan format gambar yang didukung harus ditentukan melalui investigasi teknis.
- Renderer artwork harus terpisah dari logika domain library dan playback.
- Periksa dukungan protokol gambar pada terminal.
- Jika terminal tidak mendukung tampilan gambar, gunakan fallback yang layak.

### FR-UI-04 — Identitas Visual Rust & Walnut

Tema default adalah **Rust & Walnut**.

Arah visual:
- Hangat dan organik.
- Kontras teks yang baik.
- Identitas khas Rayap Musik.
- Tidak sekadar meniru Spotify atau aplikasi musik lain.

Palet konsep awal:

- Rust: `#A64B32`
- Walnut/cream: `#D7B995`
- Charcoal: `#242320`

Palet tersebut adalah titik awal, bukan nilai final yang tidak boleh diubah. Uji keterbacaan dan kontras pada terminal yang didukung.

### FR-UI-05 — Maskot ASCII

- Sediakan maskot rayap berbasis ASCII art.
- Bentuk minimalis dan ringkas.
- Tidak boleh mengganggu navigasi atau informasi playback.
- Sediakan fallback untuk terminal sempit.

### FR-UI-06 — Tema Alternatif

Tema yang wajib tersedia:

- Rust & Walnut — default.
- Catppuccin.
- Dracula.
- Nord.

Tema harus didefinisikan melalui struktur palette yang konsisten, bukan kondisi warna tersebar di seluruh komponen UI.

Semua tema harus mempertahankan keterbacaan dan makna warna status yang konsisten.

Tema Spotify-inspired dan ayu bukan requirement v0.2.

### FR-UI-07 — Keyboard dan Help

- Shortcut ringkas dan terdokumentasi.
- Petunjuk shortcut tersedia di UI atau layar help.
- Kontrol global konsisten di semua tampilan.
- Aksi destruktif, seperti menghapus playlist, memiliki perlindungan yang proporsional.
- Keyboard adalah metode kontrol utama.

Shortcut untuk Play Mode, queue management, playlist operations, search, dan navigasi harus ditetapkan dalam rancangan input sebelum implementasi UI penuh.

## 10. Dukungan Format Audio

Semua format berikut adalah **format wajib**, bukan target opsional:

- MP3.
- M4A/AAC.
- FLAC.
- WAV.
- OGG Vorbis.
- Opus.

### Kebijakan verifikasi teknis

QA v0.1.0 menyebut Opus belum didukung oleh stack Rodio/Symphonia yang dipin. Karena Opus sekarang wajib, dukungan decoder harus diperiksa sebelum implementasi audio v0.2 dinyatakan selesai.

Coding agent wajib:

1. Memeriksa versi crate, fitur decoder, kombinasi container/codec, backend Linux, dan kebutuhan library sistem.
2. Membuat matriks dukungan untuk keenam format.
3. Mengidentifikasi pendekatan teknis untuk format yang belum didukung.
4. Menguji file nyata dari setiap format yang tersedia.
5. Melaporkan keterbatasan, pilihan solusi, risiko, dan rekomendasi.
6. Meminta persetujuan sebelum mengubah dependency inti, mengganti backend audio, menambahkan proses eksternal, atau mengubah arsitektur.
7. Tidak menyatakan format didukung hanya karena aplikasi berhasil dikompilasi.

Jika format wajib belum dapat diputar dengan stack saat ini, hentikan implementasi bagian yang terdampak dan laporkan gap kepada product owner.

Penanganan file unsupported secara aman tetap diperlukan, tetapi bukan pengganti kewajiban mendukung semua format sebelum release v0.2 dinyatakan selesai.

## 11. Prinsip Arsitektur

Gunakan struktur modular yang sesuai dengan aplikasi yang sudah ada. Pembagian berikut adalah batas tanggung jawab konseptual, bukan perintah untuk membuat file baru bagi setiap komponen.

### Application / App State
- Mengkoordinasikan state aplikasi dan aksi pengguna.
- Menentukan perubahan state dan meneruskan perintah ke domain yang sesuai.

### Library Domain
- Track, album, artist, folder, metadata, sorting, dan pencarian.

### Indexer / Scanner
- Full scan, manual rescan, filesystem watcher, incremental update, unavailable detection.

### Persistence
- Database, schema, migrations, dan preferensi.

### Playlist Domain
- CRUD, reorder, duplicate, sorting, dan M3U import/export.

### Queue Domain
- Queue aktif, urutan asli, urutan shuffle, insert/remove, dan repeat.

### Playback Engine
- Audio decoding, output, volume, progress, end-of-track, dan playback error.

### Artwork
- Embedded artwork, gambar folder, terminal capability, scaling, dan placeholder.

### TUI Presentation
- Library views, queue, Play Mode, search, theme rendering, keyboard help.

### Kontrak antarmodul

- Domain logic tidak boleh bergantung pada Ratatui atau Crossterm.
- UI membaca state dan mengirim action.
- UI tidak mengelola decoder atau audio secara langsung.
- Library/indexer mengembalikan data terstruktur, bukan teks siap-render.
- Query database tidak tersebar di seluruh komponen UI.
- Queue dan playlist merupakan konsep berbeda.
- Playback state tidak diduplikasi per tampilan.
- Gunakan tipe dan abstraksi sesederhana mungkin.
- Jangan membuat plugin system, service layer berlebihan, atau framework internal tanpa kebutuhan nyata.

## 12. Non-Functional Requirements

### NFR-01 — Responsivitas

- UI tetap menerima input selama playback.
- Indexing dan filesystem watcher tidak membekukan TUI dalam waktu lama.
- Pekerjaan filesystem, metadata, dan database yang berpotensi lambat harus dipisahkan dari event loop UI jika diperlukan.
- Hindari asynchronous runtime baru tanpa justifikasi teknis.

### NFR-02 — Integritas Data

- Playlist dan indeks bertahan setelah aplikasi ditutup.
- Import/export dan database migration menangani error secara jelas.
- Jangan menghapus database secara diam-diam.
- Perubahan filesystem selama scan harus ditangani secara aman.

### NFR-03 — Privasi

- Semua fitur v0.2 berjalan lokal.
- Tidak ada telemetry atau analytics yang diperlukan.
- Tidak ada koneksi internet yang diperlukan untuk penggunaan normal.
- File musik asli tidak dimodifikasi.

### NFR-04 — Maintainability

- Pertahankan modular monolith dalam satu aplikasi.
- Tambahkan dependency hanya jika dibutuhkan.
- Kode diformat dan diuji.
- Tambahkan unit test untuk aturan domain.
- Jangan menonaktifkan test demi membuat build berhasil.

### NFR-05 — Skalabilitas Library

- Desain indexing agar direktori besar tidak membekukan UI.
- Jangan menjanjikan angka performa tanpa benchmark.
- Coding agent harus mengidentifikasi bottleneck dan mengusulkan pengukuran yang realistis.
- Tambahkan cache atau optimisasi berdasarkan kebutuhan yang terukur.

## 13. Acceptance Criteria

### Library dan Persistence

- **AC-01:** Root library terakhir dipulihkan ketika aplikasi dibuka kembali.
- **AC-02:** Aplikasi tidak memulai playback otomatis saat startup.
- **AC-03:** Scan mencakup root dan seluruh subdirektori tanpa mengikuti symlink direktori.
- **AC-04:** Indeks dan playlist tersimpan secara lokal dan bertahan antar-sesi.
- **AC-05:** Manual rescan menangani file/folder baru, berubah, rename, dan terhapus.
- **AC-06:** Perubahan filesystem diproses otomatis tanpa restart aplikasi.
- **AC-07:** Track hilang ditandai unavailable dan dapat dibersihkan dari indeks saat rescan.
- **AC-08:** Metadata kosong menggunakan fallback yang konsisten, termasuk judul berbasis nama file.
- **AC-09:** File dengan path berbeda tetap menjadi entri berbeda meskipun kontennya sama.
- **AC-10:** Album dengan nama sama digabung dan label artist menggunakan artist track pertama berdasarkan urutan deterministik.
- **AC-11:** Album tracks diurutkan berdasarkan nomor track, dengan fallback nama file.
- **AC-12:** Global search menemukan songs, albums, artists, dan playlists secara lokal.

### Playback dan Queue

- **AC-13:** Play Now mengganti queue dan memulai koleksi terpilih.
- **AC-14:** Play Next satu track menyisipkan satu lagu; Play Next koleksi menyisipkan seluruh koleksi setelah track aktif.
- **AC-15:** Add to Queue menambahkan track/koleksi ke akhir queue.
- **AC-16:** Track dapat dihapus dari queue tanpa menghapus file asli atau mengubah playlist.
- **AC-17:** Shuffle ON mengacak seluruh queue.
- **AC-18:** Shuffle OFF mengembalikan urutan asli tanpa mengganti track aktif.
- **AC-19:** Repeat Off, Repeat All, dan Repeat One berfungsi sesuai definisi.
- **AC-20:** Track rusak/unavailable ditangani tanpa crash atau loop tanpa akhir.

### Playlist

- **AC-21:** Pengguna dapat membuat, rename, menghapus, duplicate, dan membuka playlist.
- **AC-22:** Pengguna dapat menambah/menghapus track, reorder, dan sorting metadata.
- **AC-23:** Seluruh isi album, artist, atau folder dapat ditambahkan ke playlist.
- **AC-24:** M3U import/export mempertahankan urutan sejauh didukung format dan melaporkan path yang tidak tersedia.
- **AC-25:** Track yang hilang dihapus otomatis dari playlist tanpa mengubah file asli.

### UI, Artwork, dan Format

- **AC-26:** UI memiliki Songs, Albums, Artists, Playlists, dan global search.
- **AC-27:** Play Mode layar penuh dapat dibuka/tutup melalui shortcut tanpa mereset playback.
- **AC-28:** Artwork mengikuti urutan embedded art, gambar folder, lalu placeholder Rayap Musik.
- **AC-29:** Tema Rust & Walnut, Catppuccin, Dracula, dan Nord tersedia dan terbaca.
- **AC-30:** Maskot ASCII rayap memiliki fallback untuk terminal sempit.
- **AC-31:** Shortcut utama terdokumentasi dan tersedia di UI/help.
- **AC-32:** MP3, M4A/AAC, FLAC, WAV, OGG Vorbis, dan Opus telah diverifikasi melalui playback nyata.
- **AC-33:** Build, formatting, Clippy, dan automated tests relevan berhasil dijalankan dan hasil aktual dilaporkan.
- **AC-34:** Terminal dipulihkan dengan aman ketika aplikasi keluar.

## 14. Strategi Pengujian

### Unit tests

- Metadata fallback.
- Parsing track number dan sorting.
- Pengelompokan album/artist dan label album gabungan.
- Identitas track berdasarkan path.
- Search matching.
- Queue replacement, Play Next, Add to Queue, dan remove.
- Shuffle ON/OFF dengan track aktif tetap sama.
- Repeat Off/All/One.
- Interaksi shuffle dan repeat.
- Playlist CRUD, duplicate, reorder, dan sorting.
- Unavailable track dan penghapusan referensi playlist.
- Pemulihan preferensi dan validasi state.

### Integration tests

- Scan fixture direktori bertingkat.
- Database persistence setelah close/reopen.
- Rescan dan perubahan filesystem.
- M3U import/export, path relatif/absolut, path hilang, dan urutan.
- Global search lintas jenis entitas.
- Queue dari track, album, artist, folder, dan playlist.
- Error database, folder tidak terbaca, file rusak, dan track yang dihapus ketika sesi berjalan.
- Fallback terminal dan perpindahan tema jika dapat diuji secara otomatis.

### Manual tests

- Putar file nyata untuk setiap format wajib: MP3, M4A/AAC, FLAC, WAV, OGG Vorbis, Opus.
- Verifikasi pause/resume, volume, next/previous, shuffle, repeat, dan auto-next.
- Verifikasi artwork pada terminal yang mendukungnya dan fallback pada terminal yang tidak mendukungnya.
- Uji watcher ketika file ditambahkan, di-rename, diubah, dan dihapus.
- Tutup dan buka aplikasi untuk memastikan root, indeks, playlist, dan preferensi dipulihkan tanpa auto-play.
- Verifikasi UI tetap responsif pada koleksi besar.
- Verifikasi terminal dipulihkan setelah keluar normal dan setelah error yang dapat ditangani.

Build yang berhasil bukan bukti bahwa format audio benar-benar dapat diputar. Hasil test harus membedakan PASS, FAIL, BLOCKED, dan NOT TESTED.

## 15. Tahapan Implementasi

Urutan berikut adalah rencana usulan. Jangan mengerjakan semua milestone sekaligus. Selesaikan, laporkan, dan tunggu persetujuan setelah setiap milestone.

### Milestone 0 — Repository Audit dan Technical Spike

- Periksa source aktual, struktur modul, tests, dependency, dan Git status.
- Identifikasi gap antara v0.1.0 dan PRD v0.2.
- Verifikasi dukungan keenam format audio wajib, terutama Opus.
- Bandingkan opsi database lokal.
- Evaluasi filesystem watcher dan pendekatan artwork.
- Laporkan perubahan dependency/arsitektur yang mungkin dibutuhkan.
- Jangan mengubah dependency inti atau arsitektur sebelum persetujuan.

**Exit criteria:** audit, matriks dukungan format, rekomendasi teknis, risiko, dan rencana milestone disetujui.

### Milestone 1 — Domain Models dan Persistence

- Rancang model track, album, artist, library, queue, playlist, dan preferences.
- Implementasikan database dan migration yang telah disetujui.
- Pastikan data bertahan antar-sesi.
- Pastikan startup tidak auto-play.

**Exit criteria:** test persistence dan migration lulus.

### Milestone 2 — Indexer, Scan, dan Filesystem Watcher

- Implementasikan full scan dan manual rescan.
- Implementasikan automatic watcher dan incremental update.
- Implementasikan unavailable detection serta cleanup indeks.
- Pastikan indexing tidak membekukan TUI.

**Exit criteria:** indeks konsisten pada fixture dan skenario perubahan filesystem yang diuji.

### Milestone 3 — Library Views dan Search

- Implementasikan Songs, Albums, Artists, dan global search.
- Implementasikan metadata fallback dan aturan pengelompokan album.
- Implementasikan sorting deterministik.
- Hubungkan aksi playback dari library.

**Exit criteria:** acceptance criteria library dan search terpenuhi.

### Milestone 4 — Queue, Shuffle, Repeat, dan Playback

- Implementasikan Play Now, Play Next, Add to Queue, dan remove.
- Implementasikan shuffle dan pemulihan urutan asli.
- Implementasikan Repeat Off, Repeat All, dan Repeat One.
- Verifikasi dukungan seluruh format audio wajib menggunakan pendekatan yang disetujui.
- Implementasikan error recovery.

**Exit criteria:** test queue/playback lulus dan seluruh format wajib telah diverifikasi.

### Milestone 5 — Playlist dan M3U

- Implementasikan playlist CRUD.
- Implementasikan add/remove, reorder, duplicate, dan sorting.
- Implementasikan M3U import/export.
- Implementasikan penanganan track unavailable.

**Exit criteria:** persistence, ordering, dan import/export teruji.

### Milestone 6 — Artwork, Play Mode, Tema, dan Polish

- Implementasikan Play Mode layar penuh.
- Implementasikan embedded artwork, gambar folder, dan placeholder.
- Implementasikan maskot ASCII.
- Implementasikan Rust & Walnut, Catppuccin, Dracula, dan Nord.
- Lengkapi shortcut dan help.

**Exit criteria:** UX utama teruji pada terminal yang didukung dan fallback bekerja.

### Milestone 7 — Hardening dan Release Review

- Jalankan seluruh test suite, formatter, Clippy, dan build.
- Uji semua format audio wajib melalui playback nyata.
- Uji persistence, watcher, error recovery, dan terminal lifecycle.
- Periksa performa, README, dan known limitations.
- Laporkan status setiap acceptance criterion.

**Exit criteria:** acceptance criteria terpenuhi, atau gap yang tersisa dilaporkan dan disetujui secara eksplisit sebelum release.

## 16. Keputusan Produk yang Sudah Dikunci

Keputusan berikut berasal dari requirement discovery dan tidak boleh diubah tanpa persetujuan:

1. Lokal sekarang; dukungan server mungkin di masa depan, bukan scope v0.2.
2. TUI sekarang; kemungkinan GUI masa depan tidak boleh mengharuskan penulisan ulang logika inti.
3. Penggunaan pribadi.
4. Satu root library.
5. Manual rescan dan automatic filesystem watcher.
6. Metadata read-only; tidak mengubah tag file audio.
7. Startup membuka Library, memulihkan preferensi, dan tidak auto-play.
8. Library memiliki Songs, Albums, Artists, dan Playlists.
9. Search global mencakup songs, albums, artists, dan playlists.
10. Play Now mengganti queue.
11. Play Next untuk satu track berarti satu lagu; untuk album/artist/folder berarti seluruh koleksi.
12. Shuffle ON mengacak seluruh queue.
13. Shuffle OFF memulihkan urutan asli tanpa mengganti track aktif.
14. Repeat terdiri dari Off, Repeat All, dan Repeat One.
15. Playlist disimpan dalam format internal; M3U digunakan untuk import/export.
16. Playlist mendukung operasi dasar, reorder, duplicate, dan sorting metadata.
17. Track unavailable dihapus otomatis dari playlist; indeks menandai unavailable dan rescan dapat membersihkan entri.
18. File dengan path berbeda tetap menjadi entri berbeda.
19. Album dengan nama sama digabung dan label artist menggunakan artist track pertama berdasarkan urutan deterministik.
20. Track album diurutkan berdasarkan nomor track, dengan fallback nama file.
21. Artwork mengikuti urutan embedded art, gambar folder, placeholder Rayap Musik.
22. Tema default Rust & Walnut.
23. Tema alternatif Catppuccin, Dracula, dan Nord.
24. Maskot ASCII rayap minimalis.
25. Seluruh enam format audio yang tercantum di Section 10 wajib didukung.
26. Keterbatasan format audio harus dilaporkan; perubahan dependency atau arsitektur memerlukan persetujuan.

## 17. Keputusan Teknis yang Belum Dikunci

Coding agent harus mengusulkan dan menjelaskan hal-hal berikut sebelum implementasi terkait:

- Pilihan database embedded dan pendekatan migration.
- Penyimpanan preferensi: database atau file konfigurasi.
- Library watcher dan strategi debounce.
- Decoder/backend yang mendukung keenam format audio wajib.
- Strategi identitas serta referensi playlist saat impor M3U.
- Nama file gambar folder dan format artwork yang didukung.
- Protokol gambar terminal dan fallback.
- Sorting default untuk Songs, Albums, Artists, dan search results.
- Perilaku Play Next jika belum ada track aktif.
- Perilaku Play Now ketika sumber kosong.
- Kebijakan urutan queue ketika track ditambahkan/dihapus saat shuffle aktif.
- Preferensi playback mana yang dipulihkan antar-sesi.
- Target performa dan ukuran koleksi untuk pengujian.

Keputusan teknis yang tidak memengaruhi requirement produk dapat diusulkan oleh coding agent, tetapi keputusan yang mengubah perilaku pengguna, menambah layanan eksternal, mengganti dependency inti, atau mengubah arsitektur harus mendapat persetujuan terlebih dahulu.

---

## Definisi Selesai

Rayap Musik v0.2 dinyatakan selesai ketika aplikasi dapat mengelola library musik lokal secara persisten, mendeteksi perubahan filesystem, mencari dan mengelompokkan koleksi, mengelola queue dan playlist, memutar seluruh format audio wajib yang telah diverifikasi, menampilkan artwork serta Play Mode, menyediakan tema dan shortcut yang ditentukan, dan tetap responsif serta aman digunakan.

Semua acceptance criteria harus memiliki status verifikasi yang jelas. Jangan menyatakan release siap jika format audio wajib belum terverifikasi atau masih ada gap requirement yang belum disetujui.
