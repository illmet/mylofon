package models

type Stats struct {
	TotalUsers    int `json:"total_users"`
	TotalPosts    int `json:"total_posts"`
	TotalComments int `json:"total_comments"`
	TotalLikes    int `json:"total_likes"`
	UsersToday    int `json:"users_today"`
	PostsToday    int `json:"posts_today"`
}

type StatsRepository struct {
	users *UserRepository
	posts *PostRepository
}

func NewStatsRepository(users *UserRepository, posts *PostRepository) *StatsRepository {
	return &StatsRepository{users: users, posts: posts}
}

func (r *StatsRepository) GetAll() (*Stats, error) {
	stats := &Stats{}
	var err error

	stats.TotalUsers, err = r.users.CountTotal()
	if err != nil {
		return nil, err
	}

	stats.TotalPosts, err = r.posts.CountTotal()
	if err != nil {
		return nil, err
	}

	stats.TotalComments, err = r.posts.CountComments()
	if err != nil {
		return nil, err
	}

	stats.TotalLikes, err = r.posts.CountLikes()
	if err != nil {
		return nil, err
	}

	stats.UsersToday, err = r.users.CountToday()
	if err != nil {
		return nil, err
	}

	stats.PostsToday, err = r.posts.CountToday()
	if err != nil {
		return nil, err
	}

	return stats, nil
}
