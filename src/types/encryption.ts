// 加密进度（用于前端显示）
export interface EncryptionProgress {
  current: number;
  total: number;
  currentFile: string;
}

// 加密项目参数
export interface EncryptProjectParams {
  project_path: string;
  password: string;
  confirm_password: string;
}

// 解密项目参数
export interface DecryptProjectParams {
  project_path: string;
  password: string;
}

// 修改密码参数
export interface ChangePasswordParams {
  project_path: string;
  old_password: string;
  new_password: string;
  confirm_password: string;
}

// 禁用加密参数
export interface DisableEncryptionParams {
  project_path: string;
  password: string;
}

// 验证密码参数
export interface VerifyPasswordParams {
  project_path: string;
  password: string;
}
