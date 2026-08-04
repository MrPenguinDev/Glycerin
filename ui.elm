-- Glycerin Browser UI - Phase 11: Ribbon Interface with WASM Text Layout Support
-- Modern browser chrome with tabs, address bar, status bar, and extension panel

module Main exposing (..)

import Browser
import Html exposing (Html, button, div, input, text, span)
import Html.Attributes exposing (class, placeholder, value, style)
import Html.Events exposing (onClick, onInput, onSubmit)
import Json.Encode as Encode
import Http


-- MODEL

type alias Model =
    { url : String
    , searchQuery : String
    , tabs : List Tab
    , activeTab : Int
    , statusMessage : String
    , loadProgress : Float
    , ribbonExpanded : Bool
    , wasmModules : List String
    , extensions : List Extension
    }

type alias Tab =
    { id : Int
    , title : String
    , url : String
    , loading : Bool
    }

type alias Extension =
    { name : String
    , enabled : Bool
    , script : String
    }

initialModel : Model
initialModel =
    { url = ""
    , searchQuery = ""
    , tabs = [ { id = 1, title = "New Tab", url = "", loading = False } ]
    , activeTab = 1
    , statusMessage = "Ready"
    , loadProgress = 0
    , ribbonExpanded = True
    , wasmModules = []
    , extensions = []
    }


-- MSG

type Msg
    = Navigate String
    | Search String
    | InputUrl String
    | InputSearch String
    | NewTab
    | CloseTab Int
    | SwitchTab Int
    | LoadWasm String
    | LoadExtension String
    | ToggleRibbon
    | UpdateStatus String
    | SetProgress Float
    | NoOp


-- UPDATE

update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Navigate newUrl ->
            let
                updatedTabs =
                    List.map (\t -> if t.id == model.activeTab then { t | url = newUrl, loading = True } else t) model.tabs
            in
            ( { model
                | url = newUrl
                , tabs = updatedTabs
                , statusMessage = "Loading..."
              }
            , Cmd.none
            )

        Search query ->
            let
                searchUrl = "https://duckduckgo.com/html/?q=" ++ query
            in
            update (Navigate searchUrl) model

        InputUrl newUrl ->
            ( { model | url = newUrl }, Cmd.none )

        InputSearch query ->
            ( { model | searchQuery = query }, Cmd.none )

        NewTab ->
            let
                newId = (List.maximum (List.map .id model.tabs) |> Maybe.withDefault 0) + 1
                newTab = { id = newId, title = "New Tab", url = "", loading = False }
            in
            ( { model | tabs = model.tabs ++ [ newTab ], activeTab = newId }, Cmd.none )

        CloseTab tabId ->
            if List.length model.tabs <= 1 then
                ( model, Cmd.none )
            else
                let
                    filteredTabs = List.filter (\t -> t.id /= tabId) model.tabs
                    newActive = if model.activeTab == tabId then
                        List.head filteredTabs |> Maybe.map .id |> Maybe.withDefault 1
                    else
                        model.activeTab
                in
                ( { model | tabs = filteredTabs, activeTab = newActive }, Cmd.none )

        SwitchTab tabId ->
            ( { model | activeTab = tabId }, Cmd.none )

        LoadWasm path ->
            ( { model | wasmModules = path :: model.wasmModules, statusMessage = "WASM loaded: " ++ path }, Cmd.none )

        LoadExtension script ->
            let
                newExt = { name = "Custom Extension", enabled = True, script = script }
            in
            ( { model | extensions = newExt :: model.extensions, statusMessage = "Extension loaded" }, Cmd.none )

        ToggleRibbon ->
            ( { model | ribbonExpanded = not model.ribbonExpanded }, Cmd.none )

        UpdateStatus msg ->
            ( { model | statusMessage = msg }, Cmd.none )

        SetProgress progress ->
            ( { model | loadProgress = progress }, Cmd.none )

        NoOp ->
            ( model, Cmd.none )


-- VIEW

view : Model -> Html Msg
view model =
    div [ class "browser-window" ]
        [ viewRibbon model
        , viewTabBar model
        , viewViewport model
        , viewStatusBar model
        ]


viewRibbon : Model -> Html Msg
viewRibbon model =
    div [ class "ribbon", style "display" (if model.ribbonExpanded then "block" else "none") ]
        [ div [ class "ribbon-group" ]
            [ button [ onClick (Navigate model.url), class "nav-btn" ] [ text "⟳" ]
            , button [ onClick (Navigate "about:blank"), class "nav-btn" ] [ text "⌂" ]
            ]
        , div [ class "ribbon-group" ]
            [ input
                [ placeholder "Enter URL or search..."
                , value model.url
                , onInput InputUrl
                , onSubmit (Navigate model.url)
                , class "url-bar"
                ]
                []
            ]
        , div [ class "ribbon-group" ]
            [ button [ onClick NewTab, class "action-btn" ] [ text "+" ]
            , button [ onClick ToggleRibbon, class "action-btn" ] [ text "▼" ]
            ]
        , div [ class "ribbon-group" ]
            [ input
                [ placeholder "Load WASM module..."
                , onInput (\path -> LoadWasm path)
                , class "wasm-input"
                ]
                []
            ]
        ]


viewTabBar : Model -> Html Msg
viewTabBar model =
    div [ class "tab-bar" ]
        (List.map (viewTab model.activeTab) model.tabs)


viewTab : Int -> Tab -> Html Msg
viewTab activeId tab =
    div
        [ class (if tab.id == activeId then "tab active" else "tab")
        , onClick (SwitchTab tab.id)
        ]
        [ span [] [ text tab.title ]
        , button
            [ onClick (CloseTab tab.id)
            , class "close-tab"
            ]
            [ text "×" ]
        ]


viewViewport : Model -> Html Msg
viewViewport model =
    div [ class "viewport" ]
        [ if model.loadProgress > 0 && model.loadProgress < 1 then
            div [ class "progress-bar" ]
                [ div [ class "progress-fill", style "width" (String.fromFloat (model.loadProgress * 100) ++ "%") ] []
                ]
          else
            text ""
        , div [ class "content-area" ]
            [ text "Content rendering via WASM GPU layout..."
            , br [] []
            , text ("Current URL: " ++ model.url)
            , br [] []
            , text ("WASM Modules: " ++ String.join ", " model.wasmModules)
            ]
        ]


viewStatusBar : Model -> Html Msg
viewStatusBar model =
    div [ class "status-bar" ]
        [ span [ class "status-message" ] [ text model.statusMessage ]
        , span [ class "extension-count" ] [ text ("Extensions: " ++ String.fromInt (List.length model.extensions)) ]
        ]


-- MAIN

main : Program () Model Msg
main =
    Browser.sandbox
        { init = initialModel
        , update = update
        , view = view
        }
