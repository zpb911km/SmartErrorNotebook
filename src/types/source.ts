export interface Source {
  id: string
  subjectId: string | null
  book: string | null
  chapter: string | null
  knowledge: string | null
  createdAt: string
  updatedAt: string
}

export interface CreateSourceRequest {
  subjectId?: string | null
  book?: string | null
  chapter?: string | null
  knowledge?: string | null
}

export interface CreateSourceResponse {
  source: Source
}

export interface UpdateSourceRequest {
  id: string
  subjectId: string | null
  book: string | null
  chapter: string | null
  knowledge: string | null
}

export interface UpdateSourceResponse {
  source: Source
}

export interface DeleteSourceRequest {
  id: string
}

export interface DeleteSourceResponse {
  id: string
}

export interface DeleteSourcesRequest {
  ids: string[]
}

export interface DeleteSourcesResponse {
  ids: string[]
}

export interface GetSourceRequest {
  id: string
}

export interface GetSourceResponse {
  source: Source
}

export interface ListSourcesRequest {
  subjectId?: string | null
}

export interface ListSourcesResponse {
  sources: Source[]
}
