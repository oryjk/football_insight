import type {
  AddTeamBoardCommentPayload,
  CreateTeamBoardPostPayload,
  TeamBoardComment,
  TeamBoardLikeSummary,
  TeamBoardPost,
  TeamBoardViewResponse,
} from '../types/teamBoard'
import { request } from '../utils/request'

export function getTeamBoard(teamId: number): Promise<TeamBoardViewResponse> {
  return request<TeamBoardViewResponse>({
    url: `/team-boards/${teamId}`,
    auth: true,
  })
}

export function createTeamBoardPost(
  teamId: number,
  payload: CreateTeamBoardPostPayload,
): Promise<TeamBoardPost> {
  return request<TeamBoardPost>({
    url: `/team-boards/${teamId}`,
    method: 'POST',
    data: payload,
    auth: true,
  })
}

export function addTeamBoardComment(
  postId: string,
  payload: AddTeamBoardCommentPayload,
): Promise<TeamBoardComment> {
  return request<TeamBoardComment>({
    url: `/team-boards/posts/${postId}/comments`,
    method: 'POST',
    data: payload,
    auth: true,
  })
}

export function toggleTeamBoardPostLike(postId: string): Promise<TeamBoardLikeSummary> {
  return request<TeamBoardLikeSummary>({
    url: `/team-boards/posts/${postId}/likes`,
    method: 'POST',
    auth: true,
  })
}
