import React from 'react';
import styles from './button.module.css';

export const TextButton = ({ children, className, ...props }) => (
    <button className={`${styles['text']} ${className}`} {...props}>
        {children}
    </button>
);

export const MenuTextButton = ({ children, ...props }) => (
    <TextButton className={styles['menu']} {...props}>
        {children}
    </TextButton>
);
