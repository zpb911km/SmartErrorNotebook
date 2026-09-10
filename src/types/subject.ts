export interface Subject {
  id: string
  name: string
  color: string
  createdAt: string
  updatedAt: string
}

export interface CreateSubjectRequest {
  name: string
  color: string
}

export interface CreateSubjectResponse {
  subject: Subject
}

export interface UpdateSubjectRequest extends CreateSubjectRequest {
  id: string
}

export interface UpdateSubjectResponse {
  subject: Subject
}

export interface DeleteSubjectRequest {
  id: string
}

export interface DeleteSubjectResponse {
  id: string
}

export interface ListSubjectsResponse {
  subjects: Subject[]
}
