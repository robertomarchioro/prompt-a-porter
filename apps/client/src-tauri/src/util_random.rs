// Modulo util_random — generazione di bytes casuali sicuri via OS RNG.
//
// Centralizza l'accesso a `rand::rngs::SysRng` in un unico punto:
// - Isola la dipendenza dall'API fallibile di rand 0.10 (`TryRngCore`).
// - Propaga l'errore come `PapErrore::RngNonDisponibile` (opaco, no leakage).
// - Semantica fail-closed: se l'OS RNG non è disponibile l'operazione fallisce,
//   non si prosegue con un buffer non inizializzato o debole.
//
// Tutti i moduli che generano ID o salt devono usare `riempi_random` invece
// di accedere direttamente a `SysRng`.

use rand::rand_core::TryRngCore;
use rand::rngs::SysRng;

use crate::errore::PapErrore;

/// Riempie `buf` con bytes casuali provenienti dall'OS RNG.
///
/// Propaga un `PapErrore::RngNonDisponibile` opaco se l'OS RNG non è
/// accessibile — mai `unwrap()`, mai buffer parzialmente inizializzato.
pub fn riempi_random(buf: &mut [u8]) -> Result<(), PapErrore> {
    SysRng
        .try_fill_bytes(buf)
        .map_err(|_| PapErrore::RngNonDisponibile)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn riempi_random_16_bytes() {
        // Arrange
        let mut buf = [0u8; 16];

        // Act
        riempi_random(&mut buf).expect("OS RNG dovrebbe essere disponibile in test");

        // Assert: con alta probabilità almeno un byte è diverso da zero
        // (prob di tutti zero = 1/2^128 ≈ 0, test deterministico in pratica).
        assert_ne!(buf, [0u8; 16], "Il buffer non deve essere tutto zero");
    }

    #[test]
    fn riempi_random_4_bytes() {
        let mut buf = [0u8; 4];
        riempi_random(&mut buf).expect("OS RNG dovrebbe essere disponibile in test");
        // 4 byte: prob tutti zero = 1/2^32 ≈ 2.3e-10. Accettabile come test.
        // Controlliamo solo che l'operazione non panichi e ritorni Ok.
    }

    #[test]
    fn riempi_random_buffer_vuoto() {
        // Un buffer vuoto deve sempre riuscire senza errori.
        let mut buf = [0u8; 0];
        riempi_random(&mut buf).expect("Buffer vuoto deve sempre riuscire");
    }
}
