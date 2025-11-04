export type ReactionType = 'kino' | 'info' | 'slop';

export interface Post {
  id: number;
  username: string;
  content: string;
  createdAt: string;
  reactions: {
    kino: number;
    info: number;
    slop: number;
  };
  userReaction?: ReactionType | null;
}

export interface CreatePostRequest {
  username: string;
  content: string;
}

export interface AddReactionRequest {
  postId: number;
  username: string;
  reactionType: ReactionType;
}

export interface GetPostsQuery {
  limit?: number;
  offset?: number;
}
