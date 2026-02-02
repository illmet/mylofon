package services

import (
	"crypto/rand"
	"fmt"
	"math/big"

	"golang.org/x/crypto/argon2"
)

type AuthService struct {
	// Argon2id parameters - tuned for security vs performance
	time    uint32
	memory  uint32
	threads uint8
	keyLen  uint32
}

func NewAuthService() *AuthService {
	return &AuthService{
		time:    1,
		memory:  64 * 1024, // 64MB
		threads: 4,
		keyLen:  32,
	}
}

// GenerateCode creates a random numeric code of the specified length
func (s *AuthService) GenerateCode(length int) (string, error) {
	if length < 4 || length > 32 {
		return "", fmt.Errorf("code length must be between 4 and 32")
	}

	code := make([]byte, length)
	for i := 0; i < length; i++ {
		n, err := rand.Int(rand.Reader, big.NewInt(10))
		if err != nil {
			return "", fmt.Errorf("failed to generate random digit: %w", err)
		}
		code[i] = byte('0' + n.Int64())
	}
	return string(code), nil
}

// HashCode creates an Argon2id hash of the code
func (s *AuthService) HashCode(code string) (string, error) {
	salt := make([]byte, 16)
	if _, err := rand.Read(salt); err != nil {
		return "", fmt.Errorf("failed to generate salt: %w", err)
	}

	hash := argon2.IDKey([]byte(code), salt, s.time, s.memory, s.threads, s.keyLen)

	// Encode as: $argon2id$v=19$m=65536,t=1,p=4$<salt>$<hash>
	encoded := fmt.Sprintf("$argon2id$v=%d$m=%d,t=%d,p=%d$%x$%x",
		argon2.Version, s.memory, s.time, s.threads, salt, hash)

	return encoded, nil
}

// VerifyCode checks if the provided code matches the hash
func (s *AuthService) VerifyCode(code, encodedHash string) (bool, error) {
	var version int
	var memory, time uint32
	var threads uint8
	var salt, hash []byte

	// Parse the encoded hash
	_, err := fmt.Sscanf(encodedHash, "$argon2id$v=%d$m=%d,t=%d,p=%d$%x$%x",
		&version, &memory, &time, &threads, &salt, &hash)
	if err != nil {
		return false, fmt.Errorf("invalid hash format: %w", err)
	}

	// Recompute hash with same parameters
	computed := argon2.IDKey([]byte(code), salt, time, memory, threads, uint32(len(hash)))

	// Constant-time comparison
	if len(computed) != len(hash) {
		return false, nil
	}
	var diff byte
	for i := range computed {
		diff |= computed[i] ^ hash[i]
	}
	return diff == 0, nil
}

// GenerateSessionID creates a cryptographically secure session ID
func (s *AuthService) GenerateSessionID() (string, error) {
	bytes := make([]byte, 32)
	if _, err := rand.Read(bytes); err != nil {
		return "", fmt.Errorf("failed to generate session ID: %w", err)
	}
	return fmt.Sprintf("%x", bytes), nil
}
