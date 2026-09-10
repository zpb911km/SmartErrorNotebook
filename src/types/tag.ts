export interface Tag {
  id: string
  name: string
  color: string
}

export interface CreateTagRequest {
  name: string
  color: string
}

export interface CreateTagResponse {
  tag: Tag
}

export interface UpdateTagRequest extends CreateTagRequest {
  id: string
}

export interface UpdateTagResponse {
  tag: Tag
}

export interface DeleteTagRequest {
  id: string
}

export interface DeleteTagResponse {
  id: string
}

export interface ListTagsResponse {
  tags: Tag[]
}
