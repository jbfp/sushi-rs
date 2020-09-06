import { useEffect } from 'react';

export const useSubtitle = (subtitle) => {
    useEffect(() => {
        document.title = `${subtitle} - Sushi Game - おすし`;
    }, [subtitle]);
};