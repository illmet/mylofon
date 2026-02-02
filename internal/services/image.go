package services

import (
	"fmt"
	"image"
	"image/jpeg"
	_ "image/png"
	"io"
	"os"
	"path/filepath"

	"github.com/chai2010/webp"
	"github.com/nfnt/resize"
)

type ImageService struct {
	uploadDir string
	maxSize   int // Maximum dimension (width/height)
	quality   int // WebP quality (0-100)
}

func NewImageService(uploadDir string) *ImageService {
	return &ImageService{
		uploadDir: uploadDir,
		maxSize:   256,
		quality:   80,
	}
}

// SaveAvatar processes and saves an uploaded avatar image
// Returns the relative path to the saved file
func (s *ImageService) SaveAvatar(userID int64, src io.Reader) (string, error) {
	// Decode the source image
	img, _, err := image.Decode(src)
	if err != nil {
		return "", fmt.Errorf("failed to decode image: %w", err)
	}

	// Resize to square (crop center if needed, then resize)
	img = s.cropToSquare(img)
	img = resize.Resize(uint(s.maxSize), uint(s.maxSize), img, resize.Lanczos3)

	// Ensure directory exists
	avatarDir := filepath.Join(s.uploadDir, "avatars")
	if err := os.MkdirAll(avatarDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create avatar directory: %w", err)
	}

	// Save as WebP
	filename := fmt.Sprintf("%d.webp", userID)
	fullPath := filepath.Join(avatarDir, filename)

	file, err := os.Create(fullPath)
	if err != nil {
		return "", fmt.Errorf("failed to create file: %w", err)
	}
	defer file.Close()

	if err := webp.Encode(file, img, &webp.Options{Quality: float32(s.quality)}); err != nil {
		return "", fmt.Errorf("failed to encode webp: %w", err)
	}

	// Return relative path for storage in DB
	return filepath.Join("uploads", "avatars", filename), nil
}

// SaveAvatarFromFile processes an avatar from a file path
func (s *ImageService) SaveAvatarFromFile(userID int64, srcPath string) (string, error) {
	file, err := os.Open(srcPath)
	if err != nil {
		return "", fmt.Errorf("failed to open file: %w", err)
	}
	defer file.Close()

	return s.SaveAvatar(userID, file)
}

// cropToSquare crops the image to a centered square
func (s *ImageService) cropToSquare(img image.Image) image.Image {
	bounds := img.Bounds()
	width := bounds.Dx()
	height := bounds.Dy()

	if width == height {
		return img
	}

	var size, x, y int
	if width < height {
		size = width
		x = 0
		y = (height - width) / 2
	} else {
		size = height
		x = (width - height) / 2
		y = 0
	}

	// Create cropped image
	cropped := image.NewRGBA(image.Rect(0, 0, size, size))
	for i := 0; i < size; i++ {
		for j := 0; j < size; j++ {
			cropped.Set(i, j, img.At(bounds.Min.X+x+i, bounds.Min.Y+y+j))
		}
	}

	return cropped
}

// DeleteAvatar removes a user's avatar file
func (s *ImageService) DeleteAvatar(userID int64) error {
	filename := fmt.Sprintf("%d.webp", userID)
	fullPath := filepath.Join(s.uploadDir, "avatars", filename)
	
	if err := os.Remove(fullPath); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to delete avatar: %w", err)
	}
	return nil
}

// GetAvatarPath returns the full filesystem path for a user's avatar
func (s *ImageService) GetAvatarPath(userID int64) string {
	return filepath.Join(s.uploadDir, "avatars", fmt.Sprintf("%d.webp", userID))
}

// ExportAsJPEG exports an image as JPEG (fallback if WebP not supported)
func (s *ImageService) ExportAsJPEG(img image.Image, w io.Writer) error {
	return jpeg.Encode(w, img, &jpeg.Options{Quality: 85})
}
