Specyfikacja: Clipboard Parser (Spicy/Janitor/Generic)

1. Cel

Wyodrębnianie danych postaci z surowego tekstu schowka i ładowanie ich do formularza edycji po zatwierdzeniu przez użytkownika.

2. Lokalizacja w UI

Miejsce: Górna belka (Top Bar) w CentralPanel (okno edycji postaci).

Przyciski: [IMPORT] [EXPORT].

Akcja: Kliknięcie [IMPORT] otwiera modal/nakładkę z dużym polem tekstowym na surowy tekst.

3. Logika Rozpoznawania (Heurystyka)

Parser przeszukuje ciąg znaków w poszukiwaniu wzorców (case-insensitive):

Name: Tekst wyodrębniony z sekcji nagłówkowej (okolice "@user" lub pod "Back").

First Message: Blok tekstu po markerze "Greeting" lub "First Message".

Personality: Blok tekstowy lub lista klucz-wartość (age, gender, appearance itp.).

Scenario: Tekst po markerze "Scenario".

External Tags: Słowa kluczowe wyłapane z sekcji tagów strony.

4. Przepływ Pracy (Workflow)

Input: Użytkownik wkleja tekst i klika "Analyze".

Data Review (Podgląd): Program wyświetla listę pól i przypisane do nich dane w formie tekstowej.

Puste pola: Jeśli parser nie znalazł danych (np. brak Scenario w tekście), pole pozostaje puste/edytowalne.

Potwierdzenie (KRYTYCZNE): Nic nie zostaje zapisane w bazie ani zmienione w modelu selected_character, dopóki użytkownik nie kliknie przycisku "Apply to Character".

Finalizacja: Po kliknięciu "Apply", dane lądują w formularzu edycji. Użytkownik musi kliknąć główny przycisk "SAVE", aby utrwalić zmiany w bazie SQL.

5. Czyszczenie Danych

Automatyczne usuwanie zbędnych spacji, pustych linii oraz śmieci nawigacyjnych (Home, Chats, Terms of Service itp.).

Ignorowanie statystyk (token counts, like counts).

Mapowanie pól klucz-wartość (np. age: 27) bezpośrednio do tekstu w polu Personality.

6. Obsługa Tagów Zewnętrznych

Wykryte tagi są parsowane jako lista i trafiają wyłącznie do pola external_tags.