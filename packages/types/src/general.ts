import type React from 'react';

export type Ref<T> = React.RefObject<T | null> | ((instance: T | null) => void);
