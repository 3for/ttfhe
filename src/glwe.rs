use crate::{k, poly::ResiduePoly, N};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};

#[derive(Clone, Copy)]
pub struct GlweCiphertext {
    pub mask: [ResiduePoly; k],
    pub body: ResiduePoly,
}

#[derive(Clone, Copy)]
pub struct SecretKey {
    pub polys: [ResiduePoly; k],
}

impl GlweCiphertext {
    pub fn encrypt(mu: u64, sk: SecretKey) -> GlweCiphertext {
        let sigma = f64::powf(2.0, 39.0);
        let normal = Normal::new(0.0, sigma).unwrap();

        let e = normal.sample(&mut rand::thread_rng()).round() as i64;
        // let e = 0;
        let mu_star = mu.wrapping_add_signed(e);

        // mask
        let mut mask: [ResiduePoly; k] = Default::default();
        for i in 0..k {
            for j in 0..N {
                mask[i].coefs[j] = rand::random::<u64>();
            }
        }

        // correct when k = 1

        let mut body = mask[0].mul(&sk.polys[0]);

        body.add_constant_assign(mu_star as u64);

        GlweCiphertext { mask, body }
    }

    pub fn decrypt(self, sk: SecretKey) -> u64 {
        // optimized computation of the dot product for when k = 1
        let body = self.mask[0].mul(&sk.polys[0]);

        let mu_star = self.body.sub(&body);
        mu_star.coefs[0]
    }

    fn add(self, rhs: Self) -> Self {
        let mut res = GlweCiphertext::default();
        for i in 0..k {
            res.mask[i] = self.mask[i].add(rhs.mask[i]);
        }
        res.body = self.body.add(rhs.body);
        res
    }
}

impl Default for GlweCiphertext {
    fn default() -> Self {
        GlweCiphertext {
            mask: [ResiduePoly::default()], // only works with k = 1
            body: ResiduePoly::default(),
        }
    }
}

pub fn keygen() -> SecretKey {
    let mut polys = [ResiduePoly::default(); k];
    for i in 0..k {
        for j in 0..N {
            polys[i].coefs[j] = thread_rng().gen_range(0..2);
        }
    }
    SecretKey { polys }
}

pub fn encode(msg: u8) -> u64 {
    (msg as u64) << 60
}

pub fn decode(mu: u64) -> u8 {
    ((((mu >> 59) + 1) >> 1) % 16) as u8
}

#[test]
fn test_keygen_enc_dec() {
    let sk = keygen();
    for _ in 0..100 {
        let msg = thread_rng().gen_range(0..16);
        let ct = GlweCiphertext::encrypt(encode(msg), sk);
        let pt = decode(ct.decrypt(sk));
        assert!(pt == msg);
    }
}

#[test]
fn test_add() {
    let sk = keygen();
    for _ in 0..100 {
        let msg1 = thread_rng().gen_range(0..16);
        let msg2 = thread_rng().gen_range(0..16);
        let ct1 = GlweCiphertext::encrypt(encode(msg1), sk);
        let ct2 = GlweCiphertext::encrypt(encode(msg2), sk);
        let res = ct1.add(ct2);
        let pt = decode(res.decrypt(sk));
        assert!(pt == (msg1 + msg2) % 16);
    }
}
