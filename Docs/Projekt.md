# **Projekt: C.R.A.P. – *Character Repository & Archive Program***

## **1\. Cel Projektu**

Budowa lekkiej, natywnej aplikacji na Windows (Rust \+ egui) do zarządzania bazą postaci AI Roleplay i powiązanych lorebooków.

## **2\. Stack Technologiczny**

* **Język:** Rust  
* **UI:** egui (Immediate Mode GUI, natywne renderowanie bez Chromium)  
* **Baza danych:** SQLite \+ sqlx (czysty SQL z weryfikacją w czasie kompilacji)  
* **Tokenizacja:** tiktoken-rs (liczenie tokenów zgodne z GPT-4/cl100k\_base \- jako standardowy punkt odniesienia)  
* **Obrazy:** image (do obróbki/kadrowania) \+ egui\_extras (do wyświetlania)

## **3\. Model Danych (Baza SQLite)**

### **Tabela: characters**

* id: INTEGER PRIMARY KEY (Auto-increment)  
* name: TEXT (Nazwa wpisu/pliku)  
* char\_name: TEXT (Imię wewnątrz świata gry)  
* char\_title: TEXT (Tytuł/Rola)  
* personality: TEXT  
* first\_message: TEXT  
* author\_notes: TEXT (Notatki prywatne użytkownika)  
* avatar\_path: TEXT (Lokalna ścieżka do pliku)  
* created\_at: DATETIME (Automatycznie)  
* updated\_at: DATETIME (Automatycznie przy edycji)

### **Tabela: lorebooks**

* id: INTEGER PRIMARY KEY  
* title: TEXT  
* content: TEXT  
* keywords: TEXT (Oddzielone przecinkami)

### **Tabela: character\_lore\_link (Relacja wiele-do-wielu)**

* character\_id: FK  
* lore\_id: FK

## **4\. Funkcjonalności MVP**

1. **Dashboard:** Lista postaci z miniaturami (kadrowanie dynamiczne do 1:1 w UI).  
2. **Edytor:** Pola tekstowe dla wszystkich parametrów postaci.  
3. **Licznik Tokenów:** Dynamiczne odświeżanie ilości tokenów przy wpisywaniu tekstu.  
4. **Zarządzanie Obrazami:** Wybór pliku \-\> Kopiowanie do wewnętrznego folderu /data/avatars/.  
5. **Tagowanie:** Możliwość przypisania prostych tagów tekstowych.

## **5\. Architektura UI (Layout egui)**

* **Side Panel (Left):** \- Wyszukiwarka.  
  * Lista scrollowalna: \[Miniatura 1:1\] \[Imię | Tytuł\].  
  * Przycisk "+ Dodaj Nową".  
* **Central Panel:**  
  * Tabs: \[Główne dane\] \[Notatki Autora\] \[Powiązane Lore\].  
  * Edytory tekstowe (Multiline).  
  * Licznik tokenów w stopce sekcji.  
* **Top Bar:** \- Import (JSON).  
  * Export (JSON).

## **6\. Instrukcje dla AI (Antigravity)**

Przy generowaniu kodu skup się na:

* Użyciu tokio do operacji asynchronicznych na bazie danych.  
* Implementacji egui::TextureHandle do sprawnego zarządzania obrazami w pamięci.  
* Zachowaniu modularności: osobny plik dla bazy danych (db.rs), osobny dla UI (ui.rs) i modeli (models.rs).