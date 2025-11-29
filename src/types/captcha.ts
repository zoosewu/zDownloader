// CAPTCHA-related TypeScript types

export interface CaptchaChallenge {
  id: string;
  captcha_type: string;
  challenge_data: string;
  params: Record<string, string>;
  task_id: string;
  module_id: string;
  created_at: number;
}

export interface CaptchaRequiredEvent {
  challenge_id: string;
  captcha_type: string;
  challenge_data: string;
  params: Record<string, string>;
  task_id: string;
  module_id: string;
}

export interface CaptchaSolvedEvent {
  challenge_id: string;
  task_id: string;
}

export interface CaptchaCancelledEvent {
  challenge_id: string;
  task_id: string;
}

export interface AuthCredentials {
  username: string;
  password: string;
  extra?: Record<string, string>;
}
