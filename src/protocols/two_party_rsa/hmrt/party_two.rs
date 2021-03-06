use crate::protocols::two_party_rsa::hmrt::gen_ddh_containers;
use crate::protocols::two_party_rsa::hmrt::party_one::PartyOneCandidateGenerationFirstMsg;
use crate::protocols::two_party_rsa::hmrt::party_one::PartyOneCandidateGenerationSecondMsg;
use crate::protocols::two_party_rsa::hmrt::party_one::PartyOneCandidateGenerationThirdMsg;
use crate::protocols::two_party_rsa::hmrt::party_one::PartyOneKeySetupFirstMsg as KeySetupFirstMsgPartyOne;
use crate::protocols::two_party_rsa::hmrt::CiphertextPair;
use crate::protocols::two_party_rsa::CANDIDATE_BIT_LENGTH;
use crate::protocols::two_party_rsa::PAILLIER_MODULUS;
use crate::utlities::ddh_proof::DDHProof;
use crate::utlities::ddh_proof::DDHStatement;
use crate::utlities::ddh_proof::DDHWitness;
use crate::utlities::ddh_proof::NISigmaProof;
use crate::utlities::dlog_proof::DLogProof;
use crate::utlities::dlog_proof::ProveDLog;
use crate::utlities::dlog_proof::Statement as DLogStatement;
use crate::utlities::dlog_proof::Witness as DLogWitness;
use crate::utlities::elgamal_enc_proof::HomoELGamalProof;
use crate::utlities::elgamal_enc_proof::HomoElGamalStatement;
use crate::utlities::elgamal_enc_proof::HomoElGamalWitness;
use crate::utlities::mod_proof::ModProof;
use crate::utlities::mod_proof::ModStatement;
use crate::utlities::mod_proof::ModWitness;
use crate::utlities::range_proof::RangeProof;
use crate::utlities::range_proof::Statement as BoundStatement;
use crate::utlities::range_proof::Witness as BoundWitness;
use crate::TwoPartyRSAError;
use curv::arithmetic::traits::Modulo;
use curv::arithmetic::traits::Samplable;
use curv::BigInt;
use elgamal::rfc7919_groups::SupportedGroups;
use elgamal::ElGamalCiphertext;
use elgamal::ElGamalKeyPair;
use elgamal::ElGamalPP;
use elgamal::ElGamalPrivateKey;
use elgamal::ElGamalPublicKey;
use elgamal::ExponentElGamal;
use paillier::traits::KeyGeneration;
use paillier::DecryptionKey;
use paillier::EncryptionKey;
use paillier::Paillier;
use zk_paillier::zkproofs::NICorrectKeyProof;

//TODO: add zeroize if needed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyTwoKeySetup {
    pub local_paillier_pubkey: EncryptionKey,
    pub local_elgamal_puubkey: ElGamalPublicKey,
    pub remote_paillier_pubkey: EncryptionKey,
    pub remote_elgamal_pubkey: ElGamalPublicKey,
    pub joint_elgamal_pubkey: ElGamalPublicKey,
    private: PartyTwoPrivate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeySetupFirstMsg {
    pub ek: EncryptionKey,
    pub pk: ElGamalPublicKey,
    pub correct_key_proof: NICorrectKeyProof,
    pub dlog_proof: DLogProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyTwoPrivate {
    dk: DecryptionKey,
    sk: ElGamalPrivateKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyTwoCandidateGeneration {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyTwoCandidateGenerationFirstMsg {
    pub c_i: ElGamalCiphertext,
    pub pi_enc: HomoELGamalProof,
    pub pi_bound: RangeProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyTwoCandidateGenerationSecondMsg {
    pub pi_mod: ModProof,
    pub c_1_alpha: ElGamalCiphertext,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyTwoCandidateGenerationThirdMsg {
    pub proof_alpha: DDHProof,
    pub proof_alpha_tilde: DDHProof,
    pub c_alpha_random: ElGamalCiphertext,
    pub c_alpha_tilde_random: ElGamalCiphertext,
    pub partial_dec_c_alpha: BigInt,
    pub partial_dec_c_alpha_tilde: BigInt,
    pub ddh_proof_alpha: DDHProof,
    pub ddh_proof_alpha_tilde: DDHProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyTwoCandidateWitness {
    pub p_1: BigInt,
    pub r_1: BigInt,
}

impl PartyTwoKeySetup {
    pub fn gen_local_keys_and_first_message_to_party_one() -> (KeySetupFirstMsg, PartyTwoPrivate) {
        let pp = ElGamalPP::generate_from_rfc7919(SupportedGroups::FFDHE2048);
        let keypair = ElGamalKeyPair::generate(&pp);
        let witness = DLogWitness {
            x: keypair.sk.x.clone(),
        };
        let dlog_proof = DLogProof::prove(&witness, &pp);

        let (ek_new, dk_new) = Paillier::keypair_with_modulus_size(PAILLIER_MODULUS).keys();
        let correct_key_proof = NICorrectKeyProof::proof(&dk_new);

        let party_two_private = PartyTwoPrivate {
            dk: dk_new,
            sk: keypair.sk,
        };
        (
            KeySetupFirstMsg {
                ek: ek_new,
                pk: keypair.pk.clone(),
                correct_key_proof,
                dlog_proof,
            },
            party_two_private,
        )
    }

    pub fn verify_party_one_first_message_and_output_party_two_keys(
        party_one_first_message: &KeySetupFirstMsgPartyOne,
        party_two_first_message: &KeySetupFirstMsg,
        party_two_private: PartyTwoPrivate,
    ) -> Result<Self, TwoPartyRSAError> {
        let dlog_statement = DLogStatement {
            h: party_one_first_message.pk.h.clone(),
        };

        match party_one_first_message
            .dlog_proof
            .verify(&dlog_statement, &party_two_first_message.pk.pp)
        {
            Ok(()) => {
                match party_one_first_message
                    .correct_key_proof
                    .verify(&party_one_first_message.ek)
                {
                    Ok(()) => Ok(PartyTwoKeySetup {
                        local_paillier_pubkey: party_two_first_message.ek.clone(),
                        local_elgamal_puubkey: party_two_first_message.pk.clone(),
                        remote_paillier_pubkey: party_one_first_message.ek.clone(),
                        remote_elgamal_pubkey: party_one_first_message.pk.clone(),
                        joint_elgamal_pubkey: party_two_first_message
                            .pk
                            .add(&party_one_first_message.pk)
                            .unwrap(),
                        private: party_two_private,
                    }),
                    Err(_) => Err(TwoPartyRSAError::InvalidPaillierKey),
                }
            }
            Err(_) => Err(TwoPartyRSAError::InvalidElGamalKey),
        }
    }
}

impl PartyTwoCandidateGeneration {
    pub fn generate_shares_of_candidate(
        keys: &PartyTwoKeySetup,
    ) -> (
        PartyTwoCandidateWitness,
        PartyTwoCandidateGenerationFirstMsg,
    ) {
        let share_bit_size: usize = CANDIDATE_BIT_LENGTH / 2 - 2;
        let p_i = BigInt::sample(share_bit_size);
        let r_i = BigInt::sample_below(&keys.joint_elgamal_pubkey.pp.q);

        let c_i = ExponentElGamal::encrypt_from_predefined_randomness(
            &p_i,
            &keys.joint_elgamal_pubkey,
            &r_i,
        )
        .unwrap();
        let enc_witness = HomoElGamalWitness {
            r: r_i.clone(),
            m: p_i.clone(),
        };
        let enc_statement = HomoElGamalStatement {
            pk: keys.joint_elgamal_pubkey.clone(),
            ciphertext: c_i.clone(),
        };
        let bound_witness = BoundWitness {
            x: p_i.clone(),
            r: r_i.clone(),
        };
        let bound_statement = BoundStatement {
            pk: keys.joint_elgamal_pubkey.clone(),
            // TODO: in current range proof this will give some slack such that it is possible that
            // a prover chose 2^(N/2-2)<x< 1/3 * 2^(N/2).
            range: BigInt::from(2).pow((CANDIDATE_BIT_LENGTH / 2) as u32),
            ciphertext: c_i.clone(),
            sec_param: 120, //TODO : parameterize
            kapa: 100,      //TODO : parameterize
        };

        let enc_proof = HomoELGamalProof::prove(&enc_witness, &enc_statement);
        let bound_proof = RangeProof::prove(&bound_witness, &bound_statement).unwrap(); // TODO: handle error properly

        (
            PartyTwoCandidateWitness { p_1: p_i, r_1: r_i },
            PartyTwoCandidateGenerationFirstMsg {
                c_i,
                pi_enc: enc_proof,
                pi_bound: bound_proof,
            },
        )
    }
    pub fn verify_party_one_first_message_and_normalize_ciphertexts(
        keys: &PartyTwoKeySetup,
        party_one_first_message: &PartyOneCandidateGenerationFirstMsg,
        party_two_first_message: &PartyTwoCandidateGenerationFirstMsg,
    ) -> Result<CiphertextPair, TwoPartyRSAError> {
        let enc_statement = HomoElGamalStatement {
            pk: keys.joint_elgamal_pubkey.clone(),
            ciphertext: party_one_first_message.c_i.clone(),
        };

        let bound_statement = BoundStatement {
            pk: keys.joint_elgamal_pubkey.clone(),
            range: BigInt::from(2).pow((CANDIDATE_BIT_LENGTH / 2) as u32),
            ciphertext: party_one_first_message.c_i.clone(),
            sec_param: 120,
            kapa: 100,
        };

        match party_one_first_message
            .pi_enc
            .verify(&enc_statement)
            .is_ok()
            && party_one_first_message
                .pi_bound
                .verify(&bound_statement)
                .is_ok()
        {
            true => {
                let c_party_one_mul_4 =
                    ExponentElGamal::mul(&party_one_first_message.c_i, &BigInt::from(4));
                let c_party_two_mul_4 =
                    ExponentElGamal::mul(&party_two_first_message.c_i, &BigInt::from(4));
                Ok(CiphertextPair {
                    c0: ExponentElGamal::add(
                        &c_party_one_mul_4,
                        &ExponentElGamal::encrypt_from_predefined_randomness(
                            &BigInt::from(3),
                            &keys.joint_elgamal_pubkey,
                            &BigInt::zero(),
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                    c1: c_party_two_mul_4,
                })
            }
            false => Err(TwoPartyRSAError::CandidateGenerationEncError),
        }
    }

    pub fn trial_division_prepare_c_alpha(
        alpha: &BigInt,
        keys: &PartyTwoKeySetup,
        c: &CiphertextPair,
        w: &PartyTwoCandidateWitness,
    ) -> Result<PartyTwoCandidateGenerationSecondMsg, TwoPartyRSAError> {
        // update witness:
        let p_1 = BigInt::mod_mul(&w.p_1, &BigInt::from(4), &keys.joint_elgamal_pubkey.pp.q);
        let r_1 = BigInt::mod_mul(&w.r_1, &BigInt::from(4), &keys.joint_elgamal_pubkey.pp.q);

        let p_1_mod_alpha = p_1.mod_floor(alpha);
        let r_1_alpha = BigInt::sample_below(&keys.joint_elgamal_pubkey.pp.q);
        let c_1_alpha = ExponentElGamal::encrypt_from_predefined_randomness(
            &p_1_mod_alpha,
            &keys.joint_elgamal_pubkey,
            &r_1_alpha,
        )
        .unwrap();

        let mod_statement = ModStatement {
            c: c.c1.clone(),
            c_prime: c_1_alpha.clone(),
            modulus_p: alpha.clone(),
            upper_bound_m: BigInt::from(2).pow((CANDIDATE_BIT_LENGTH / 2) as u32), // n/2 instead of n/2-2 as is written in the paper : we suspect paper has a typo and do not consider the fact that ciphertexts and plaintext are scaled by mul4
            pk: keys.joint_elgamal_pubkey.clone(),
        };

        let mod_witness = ModWitness {
            r_a: r_1,
            a: p_1,
            r_b: r_1_alpha,
            b: p_1_mod_alpha,
        };

        let proof = ModProof::prove(&mod_witness, &mod_statement);

        match proof {
            Ok(_) => Ok(PartyTwoCandidateGenerationSecondMsg {
                pi_mod: proof.unwrap(),
                c_1_alpha,
            }),
            Err(_) => Err(TwoPartyRSAError::InvalidModProof),
        }
    }

    pub fn verify_party_one_second_message_and_partial_decrypt(
        party_one_second_message: &PartyOneCandidateGenerationSecondMsg,
        party_two_second_message: &PartyTwoCandidateGenerationSecondMsg,
        alpha: &BigInt,
        keys: &PartyTwoKeySetup,
        c: &CiphertextPair,
    ) -> Result<
        (
            PartyTwoCandidateGenerationThirdMsg,
            ElGamalCiphertext,
            ElGamalCiphertext,
        ),
        TwoPartyRSAError,
    > {
        let mod_statement = ModStatement {
            c: c.c0.clone(),
            c_prime: party_one_second_message.c_0_alpha.clone(),
            modulus_p: alpha.clone(),
            upper_bound_m: BigInt::from(2).pow((CANDIDATE_BIT_LENGTH / 2) as u32),
            pk: keys.joint_elgamal_pubkey.clone(),
        };
        let verify = party_one_second_message.pi_mod.verify(&mod_statement);
        if verify.is_err() {
            return Err(TwoPartyRSAError::InvalidModProof);
        };
        let c_alpha = ExponentElGamal::add(
            &party_one_second_message.c_0_alpha,
            &party_two_second_message.c_1_alpha,
        )
        .unwrap();
        // Enc(-alpha) is known to both parties therefore we use a predefined randomness known to both (r = 2)
        let enc_alpha = ExponentElGamal::encrypt_from_predefined_randomness(
            alpha,
            &keys.joint_elgamal_pubkey,
            &BigInt::from(2),
        )
        .unwrap();
        let enc_minus_alpha = ExponentElGamal::mul(&enc_alpha, &(-BigInt::one()));
        let c_alpha_tilde = ExponentElGamal::add(&c_alpha, &enc_minus_alpha).unwrap();

        // we raise each ciphertext with a secret random number
        let r_alpha = BigInt::sample_below(&keys.joint_elgamal_pubkey.pp.q);
        let r_alpha_tilde = BigInt::sample_below(&keys.joint_elgamal_pubkey.pp.q);

        let c_alpha_random = ExponentElGamal::mul(&c_alpha, &r_alpha);
        let c_alpha_tilde_random = ExponentElGamal::mul(&c_alpha_tilde, &r_alpha_tilde);

        // we use proof of DDH to prove to counter party that c_alpha_random = c_alpha ^r and same for c_alpha_tilde
        let (_, _, ddh_proof_alpha) = gen_ddh_containers(
            r_alpha,
            &c_alpha.c1,
            &c_alpha_random.c1,
            &c_alpha.c2,
            &c_alpha_random.c2,
            &keys.joint_elgamal_pubkey.pp,
        );

        let (_, _, ddh_proof_alpha_tilde) = gen_ddh_containers(
            r_alpha_tilde,
            &c_alpha_tilde.c1,
            &c_alpha_tilde_random.c1,
            &c_alpha_tilde.c2,
            &c_alpha_tilde_random.c2,
            &keys.joint_elgamal_pubkey.pp,
        );

        let dec_key_alpha = BigInt::mod_pow(
            &c_alpha_random.c1,
            &keys.private.sk.x,
            &keys.joint_elgamal_pubkey.pp.p,
        );
        let dec_key_alpha_tilde = BigInt::mod_pow(
            &c_alpha_tilde_random.c1,
            &keys.private.sk.x,
            &keys.joint_elgamal_pubkey.pp.p,
        );

        let statement_alpha = DDHStatement {
            pp: keys.joint_elgamal_pubkey.pp.clone(),
            g1: keys.joint_elgamal_pubkey.pp.g.clone(),
            h1: keys.local_elgamal_puubkey.h.clone(),
            g2: c_alpha_random.c1.clone(),
            h2: dec_key_alpha.clone(),
        };
        let witness_alpha = DDHWitness {
            x: keys.private.sk.x.clone(),
        };
        let statement_alpha_tilde = DDHStatement {
            pp: keys.joint_elgamal_pubkey.pp.clone(),
            g1: keys.joint_elgamal_pubkey.pp.g.clone(),
            h1: keys.local_elgamal_puubkey.h.clone(),
            g2: c_alpha_tilde_random.c1.clone(),
            h2: dec_key_alpha_tilde.clone(),
        };

        let proof_alpha = DDHProof::prove(&witness_alpha, &statement_alpha);
        let proof_alpha_tilde = DDHProof::prove(&witness_alpha, &statement_alpha_tilde);

        Ok((
            PartyTwoCandidateGenerationThirdMsg {
                proof_alpha,
                proof_alpha_tilde,
                c_alpha_random,
                c_alpha_tilde_random,
                partial_dec_c_alpha: dec_key_alpha,
                partial_dec_c_alpha_tilde: dec_key_alpha_tilde,
                ddh_proof_alpha,
                ddh_proof_alpha_tilde,
            },
            c_alpha,
            c_alpha_tilde,
        ))
    }

    pub fn verify_party_one_third_message_full_decrypt_and_conclude_division(
        c_alpha: &ElGamalCiphertext,
        c_alpha_tilde: &ElGamalCiphertext,
        party_one_third_message: &PartyOneCandidateGenerationThirdMsg,
        keys: &PartyTwoKeySetup,
    ) -> Result<bool, TwoPartyRSAError> {
        // check that the randomization of the ciphertexts was done properly:
        let statement_alpha_ddh = DDHStatement {
            pp: keys.joint_elgamal_pubkey.pp.clone(),
            g1: c_alpha.c1.clone(),
            h1: party_one_third_message.c_alpha_random.c1.clone(),
            g2: c_alpha.c2.clone(),
            h2: party_one_third_message.c_alpha_random.c2.clone(),
        };
        let statement_alpha_tilde_ddh = DDHStatement {
            pp: keys.joint_elgamal_pubkey.pp.clone(),
            g1: c_alpha_tilde.c1.clone(),
            h1: party_one_third_message.c_alpha_tilde_random.c1.clone(),
            g2: c_alpha_tilde.c2.clone(),
            h2: party_one_third_message.c_alpha_tilde_random.c2.clone(),
        };
        if party_one_third_message
            .ddh_proof_alpha
            .verify(&statement_alpha_ddh)
            .is_err()
        {
            return Err(TwoPartyRSAError::CandidateGenerationDecError);
        }

        if party_one_third_message
            .ddh_proof_alpha_tilde
            .verify(&statement_alpha_tilde_ddh)
            .is_err()
        {
            return Err(TwoPartyRSAError::CandidateGenerationDecError);
        }

        // verify proofs of decryption:
        let statement_alpha = DDHStatement {
            pp: keys.joint_elgamal_pubkey.pp.clone(),
            g1: keys.joint_elgamal_pubkey.pp.g.clone(),
            h1: keys.remote_elgamal_pubkey.h.clone(),
            g2: party_one_third_message.c_alpha_random.c1.clone(),
            h2: party_one_third_message.partial_dec_c_alpha.clone(),
        };
        let statement_alpha_tilde = DDHStatement {
            pp: keys.joint_elgamal_pubkey.pp.clone(),
            g1: keys.joint_elgamal_pubkey.pp.g.clone(),
            h1: keys.remote_elgamal_pubkey.h.clone(),
            g2: party_one_third_message.c_alpha_tilde_random.c1.clone(),
            h2: party_one_third_message.partial_dec_c_alpha_tilde.clone(),
        };

        if party_one_third_message
            .proof_alpha
            .verify(&statement_alpha)
            .is_err()
        {
            return Err(TwoPartyRSAError::CandidateGenerationDecError);
        }

        if party_one_third_message
            .proof_alpha_tilde
            .verify(&statement_alpha_tilde)
            .is_err()
        {
            return Err(TwoPartyRSAError::CandidateGenerationDecError);
        }

        // full decryption
        let dec_key_alpha = BigInt::mod_pow(
            &party_one_third_message.c_alpha_random.c1,
            &keys.private.sk.x,
            &keys.joint_elgamal_pubkey.pp.q,
        );
        let dec_key_alpha_tilde = BigInt::mod_pow(
            &party_one_third_message.c_alpha_tilde_random.c1,
            &keys.private.sk.x,
            &keys.joint_elgamal_pubkey.pp.q,
        );
        let dec_key_alpha_full = BigInt::mod_mul(
            &dec_key_alpha,
            &party_one_third_message.partial_dec_c_alpha,
            &keys.joint_elgamal_pubkey.pp.q,
        );
        let dec_key_alpha_tilde_full = BigInt::mod_mul(
            &dec_key_alpha_tilde,
            &party_one_third_message.partial_dec_c_alpha_tilde,
            &keys.joint_elgamal_pubkey.pp.q,
        );

        let dec_key_alpha_full_inv =
            BigInt::mod_inv(&dec_key_alpha_full, &keys.joint_elgamal_pubkey.pp.q);
        let dec_key_alpha_tilde_full_inv =
            BigInt::mod_inv(&dec_key_alpha_tilde_full, &keys.joint_elgamal_pubkey.pp.q);

        let test1 = BigInt::mod_mul(
            &party_one_third_message.c_alpha_random.c2,
            &dec_key_alpha_full_inv,
            &keys.joint_elgamal_pubkey.pp.q,
        );
        let test2 = BigInt::mod_mul(
            &party_one_third_message.c_alpha_tilde_random.c2,
            &dec_key_alpha_tilde_full_inv,
            &keys.joint_elgamal_pubkey.pp.q,
        );

        if test1 == BigInt::one() || test2 == BigInt::one() {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
