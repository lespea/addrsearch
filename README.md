# 🌍 How to Geocode Your Spreadsheet

This little tool takes a list of addresses from your spreadsheet and finds their exact map coordinates (Latitude and Longitude).

## Step 1: Prepare your Spreadsheet (Proton Sheets)
The tool needs a file format called **CSV** to work.

1.  Open your spreadsheet in **Proton Drive**.
2.  Make sure your **Address** column is the **very first column** (Column A).
3.  Go to **File** -> **Export** -> **CSV (.csv)**.
4.  Save the file to your computer.
    *   *Tip: Save it to your "Downloads" folder and name it `addresses.csv` to make the next steps easier.*

---

## Step 2: Open the Command Prompt (Windows)
To run the tool, you need to use the Windows "Command Prompt."

1.  Press the **Windows Key** on your keyboard (or click the Start button).
2.  Type the letters `cmd` and press **Enter**.
3.  A black window will appear. This is where you will type the commands.
4.  Type the following and press **Enter** to move into your Downloads folder:
    ```cmd
    cd Downloads
    ```

---

## Step 3: Run the Lookup
Now we tell the tool to process your file.

1.  Make sure the tool (`addrsearch.exe`) and your file (`addresses.csv`) are both in that same Downloads folder.
2.  In the black window, type this and press **Enter**:
    ```cmd
    addrsearch.exe addresses.csv
    ```

### What happens next?
You will see text appearing as the tool works through your list. When it's finished, it will say: `Geocoding complete. Results written to addresses_enriched.csv`.

---

## Step 4: View your Results
Look in your Downloads folder for the new file called **`addresses_enriched.csv`**.

You can upload this file back into **Proton Drive** to see the new data:
*   **matched_address**: The official address found.
*   **longitude / latitude**: The map coordinates.
*   **confidence**: How sure the map is (e.g., "exact" is perfect, "high" is very good).

---

## 💡 Advanced: Adjusting the "Search Area"
By default, the tool only looks for addresses in our local area to avoid finding duplicate street names in other states. If you need to search a different area, you can change the "Bounding Box."

You can use [this tool](https://boundingbox.klokantech.com/) to find a bounding box.

To use a different area, add `-b` followed by your four numbers in quotes:
```cmd
addrsearch.exe addresses.csv -b "-93.75,44.72,-92.84,45.35"
```

---

## 💡 Quick Tips
*   **Address in a different column?** If your address is in the second column (Column B) instead of the first, add `-c 1` to the command:
    *   `addrsearch.exe addresses.csv -c 1`
*   **No Header?** If your spreadsheet doesn't have a top row with labels (like "Address"), add:
    *   `addrsearch.exe addresses.csv --no-header`
*   **Help!** If you get stuck, just type `addrsearch.exe --help` to see all available options.
