export interface Attachment {
  id: string
  mimeType: string
  base64Data: string
  sha256: string
}

export interface CreateAttachmentRequest {
  mimeType: string
  base64Data: string
}

export interface CreateAttachmentResponse {
  id: string
}

export interface DeleteAttachmentResponse {
  id: string
}

export interface GetAttachmentResponse {
  attachment: Attachment
}
