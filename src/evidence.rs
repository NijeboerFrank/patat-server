use crypto::digest::Digest;
use crypto::sha2::Sha256;
use merkle_light::hash::Algorithm;
use merkle_light::merkle::MerkleTree;
use merkle_light::proof::Proof;
use std::hash::Hasher;
use std::iter::FromIterator;

struct PatatHashAlgorithm(Sha256);

impl PatatHashAlgorithm {
    pub fn new() -> PatatHashAlgorithm {
        PatatHashAlgorithm(Sha256::new())
    }
}

impl Default for PatatHashAlgorithm {
    fn default() -> Self {
        PatatHashAlgorithm::new()
    }
}

impl Hasher for PatatHashAlgorithm {
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.0.input(msg)
    }

    #[inline]
    fn finish(&self) -> u64 {
        unimplemented!()
    }
}

impl Algorithm<[u8; 32]> for PatatHashAlgorithm {
    #[inline]
    fn hash(&mut self) -> [u8; 32] {
        let mut h = [0u8; 32];
        self.0.result(&mut h);
        h
    }

    #[inline]
    fn reset(&mut self) {
        self.0.reset();
    }
}

pub struct EvidencePath(Vec<u8>);

impl EvidencePath {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_owned()
    }

    pub fn path(self) -> Vec<bool> {
        let message_length = ((self.0[0] as usize) << 8) + (self.0[1] as usize);
        let mut b = vec![];
        'outer: for (_, bit) in self.0[2..].into_iter().enumerate() {
            for i in (0..7).map(|i| 7 - i) {
                b.push((bit & (2_u8.pow(i))) >> i != 0);
                if b.len() == message_length {
                    break 'outer;
                }
            }
        }
        b
    }
}

impl From<Vec<bool>> for EvidencePath {
    fn from(value: Vec<bool>) -> Self {
        let mut b = vec![];
        b.insert(0, (value.len() >> 8).try_into().unwrap());
        b.insert(0, (value.len() & 0xff).try_into().unwrap());
        for (idx, bit) in value.into_iter().enumerate() {
            let byte = idx / 8;
            let shift = 7 - idx % 8;
            if idx % 8 == 0 {
                b.insert(0, 0);
            }
            b[byte + 2] |= (bit as u8) << shift;
        }
        EvidencePath(b)
    }
}

impl From<&[u8]> for EvidencePath {
    fn from(value: &[u8]) -> Self {
        EvidencePath(value.to_owned())
    }
}

pub struct EvidenceLemma(Vec<[u8; 32]>);

impl EvidenceLemma {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut b: Vec<u8> = vec![];
        for (_, hash) in self.0.into_iter().enumerate() {
            b.append(&mut hash.to_vec());
        }
        b
    }

    pub fn lemma(self) -> Vec<[u8; 32]> {
        self.0
    }
}

impl From<Vec<[u8; 32]>> for EvidenceLemma {
    fn from(value: Vec<[u8; 32]>) -> Self {
        EvidenceLemma(value)
    }
}

impl From<Vec<u8>> for EvidenceLemma {
    fn from(value: Vec<u8>) -> Self {
        let mut b: Vec<[u8; 32]> = vec![];
        let mut h = vec![];
        for (index, byte) in value.into_iter().enumerate() {
            if index % 32 == 0 && index != 0 {
                b.push(h.try_into().unwrap());
                h = vec![];
            }
            h.push(byte);
        }
        b.push(h.try_into().unwrap());
        EvidenceLemma(b)
    }
}

impl From<&[u8]> for EvidenceLemma {
    fn from(value: &[u8]) -> Self {
        let mut b: Vec<[u8; 32]> = vec![];
        let mut h = vec![];
        for (index, byte) in value.into_iter().enumerate() {
            if index % 32 == 0 && index != 0 {
                b.push(h.try_into().unwrap());
                h = vec![];
            }
            h.push(*byte);
        }
        b.push(h.try_into().unwrap());
        EvidenceLemma(b)
    }
}

pub struct EvidenceProof {
    lemma: EvidenceLemma,
    path: EvidencePath,
}

impl EvidenceProof {
    pub fn new(path: Vec<bool>, lemma: Vec<[u8; 32]>) -> Self {
        EvidenceProof {
            lemma: lemma.into(),
            path: path.into(),
        }
    }

    pub fn valid(self) -> bool {
        let proof: Proof<[u8; 32]> = Proof::new(self.lemma.lemma(), self.path.path());
        proof.validate::<PatatHashAlgorithm>()
    }
}

impl From<EvidenceProof> for Vec<u8> {
    fn from(value: EvidenceProof) -> Self {
        let mut buffer = vec![];

        let mut path_bytes = value.path.to_bytes();
        let path_length = path_bytes.len();
        let mut lemma_bytes = value.lemma.to_bytes();
        let lemma_length = lemma_bytes.len();

        buffer.push((path_length >> 8).try_into().unwrap());
        buffer.push((path_length & 0xff).try_into().unwrap());
        buffer.append(&mut path_bytes);

        buffer.push((lemma_length >> 8).try_into().unwrap());
        buffer.push((lemma_length & 0xff).try_into().unwrap());
        buffer.append(&mut lemma_bytes);
        buffer
    }
}

impl From<Vec<u8>> for EvidenceProof {
    fn from(value: Vec<u8>) -> Self {
        let path_length = ((value[0] as usize) << 8) + (value[1] as usize);

        let path_bytes = &value[2..2 + path_length];
        let path: EvidencePath = path_bytes.into();

        let lemma_length =
            ((value[2 + path_length] as usize) << 8) + (value[3 + path_length] as usize);
        let lemma_bytes = &value[4 + path_length..4 + path_length + lemma_length];
        let lemma: EvidenceLemma = lemma_bytes.into();
        EvidenceProof { lemma, path }
    }
}

impl From<&[u8]> for EvidenceProof {
    fn from(value: &[u8]) -> Self {
        let path_length = ((value[0] as usize) << 8) + (value[1] as usize);
        let path_bytes = &value[2..2 + path_length];
        let path: EvidencePath = path_bytes.into();

        let lemma_length =
            ((value[2 + path_length] as usize) << 8) + (value[3 + path_length] as usize);
        let lemma_bytes = &value[4 + path_length..4 + path_length + lemma_length];
        let lemma: EvidenceLemma = lemma_bytes.into();
        EvidenceProof { lemma, path }
    }
}

pub fn get_evidence() -> (Vec<[u8; 32]>, Vec<bool>) {
    let mut h1 = [0u8; 32];
    let mut h2 = [0u8; 32];
    let mut h3 = [0u8; 32];
    h1[0] = 0x11;
    h2[0] = 0x22;
    h3[0] = 0x33;
    let t: MerkleTree<[u8; 32], PatatHashAlgorithm> = MerkleTree::from_iter(vec![h1, h2, h3]);
    let p = t.gen_proof(0);
    (p.lemma().to_vec(), p.path().to_vec())
}
