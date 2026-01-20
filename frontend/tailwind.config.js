/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        main: '#A3E635',
        mainAccent: '#88CC14',
        bg: '#E3E3E3',
        text: '#000000',
        border: '#000000',
      },
      borderRadius: {
        base: '0px'
      },
      boxShadow: {
        base: '4px 4px 0px 0px rgba(0,0,0,1)',
      },
      fontWeight: {
        base: '500',
        heading: '800',
      },
    },
  },
  plugins: [],
}
