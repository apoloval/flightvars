#
#  This file is part of FlightVars
#  Copyright (C) 2016 Alvaro Polo
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
#  ------------
#
#  This NSIS script generates the installer for FlightVars.
#

!include "MUI2.nsh"
!include "XML.nsh"

!include "fs-plugins.nsh"

!ifndef FLIGHTVARS_VERSION
!define FLIGHTVARS_VERSION "Nightly Build"
!endif

###
# General properties
###
InstallDir "$PROGRAMFILES\FlightVars"
InstallDirRegKey HKCU "Software\FlightVars" ""
Name "FlightVars"
OutFile "FlightVars ${FLIGHTVARS_VERSION} Installer.exe"

###
# Pages
###
!define MUI_WELCOMEPAGE_TITLE_3LINES
!define MUI_WELCOMEPAGE_TITLE "Welcome to FlightVars ${FLIGHTVARS_VERSION} Setup"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "../LICENSE"
Page custom SimSelectionPage SimSelectionPageLeave
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

###
# Languages
###
!insertmacro MUI_LANGUAGE "English"

###
# Functions
###
Var /GLOBAL CheckBoxFSX
Var /GLOBAL CheckBoxP3DV1
Var /GLOBAL CheckBoxP3DV2
Var /GLOBAL CheckBoxP3DV3

Function SimSelectionPage
   !insertmacro MUI_HEADER_TEXT "Simulator selection" "Select your simulation software"
   nsDialogs::Create 1018
   Pop $0
   ${NSD_CreateLabel} 0u 0u 100% 25u "This installer is able to configure \
      FlightVars plugin for your simulator. Please choose the simulators you \
      want the plugin to be enabled for."
   Pop $0
   ${NSD_CreateCheckBox} 10u 30u 100% 10u "Microsoft Flight Simulator X"
   Pop $CheckBoxFSX
   ${NSD_CreateCheckBox} 10u 45u 100% 10u "Lockheed Martin Prepar3D v1"
   Pop $CheckBoxP3DV1
   ${NSD_CreateCheckBox} 10u 60u 100% 10u "Lockheed Martin Prepar3D v2"
   Pop $CheckBoxP3DV2
   ${NSD_CreateCheckBox} 10u 75u 100% 10u "Lockheed Martin Prepar3D v3"
   Pop $CheckBoxP3DV3
   ${NSD_CreateLabel} 0u 95u 100% 25u "Importante notice: only available \
      simulators are listed above. If you choose none, FlightVars will be \
      installed but the plugins will not be configured (you may do it manually \
      after installer finishes)."
   Pop $0

   ReadRegStr $0 HKCU "Software\Microsoft\Microsoft Games\Flight Simulator\10.0" "AppPath"
   ${If} $0 == ""
      ${NSD_AddStyle} $CheckBoxFSX ${WS_DISABLED}
   ${Else}
      ${NSD_Check} $CheckBoxFSX
   ${EndIf}
   ReadRegStr $0 HKCU "Software\LockheedMartin\Prepar3D" "AppPath"
   ${If} $0 == ""
      ${NSD_AddStyle} $CheckBoxP3DV1 ${WS_DISABLED}
   ${Else}
      ${NSD_Check} $CheckBoxP3DV1
   ${EndIf}
   ReadRegStr $0 HKCU "Software\Lockheed Martin\Prepar3D v2" "AppPath"
   ${If} $0 == ""
      ${NSD_AddStyle} $CheckBoxP3DV2 ${WS_DISABLED}
   ${Else}
      ${NSD_Check} $CheckBoxP3DV2
   ${EndIf}
   ReadRegStr $0 HKCU "Software\Lockheed Martin\Prepar3D v3" "AppPath"
   ${If} $0 == ""
      ${NSD_AddStyle} $CheckBoxP3DV3 ${WS_DISABLED}
   ${Else}
      ${NSD_Check} $CheckBoxP3DV3
   ${EndIf}

   nsDialogs::Show
FunctionEnd

Function SimSelectionPageLeave
   ${NSD_GetState} $CheckBoxFSX $0
   ${If} $0 == ${BST_CHECKED}
      StrCpy $OAC_InstallPluginFSX yes
   ${Else}
      StrCpy $OAC_InstallPluginFSX no
   ${EndIf}

   ${NSD_GetState} $CheckBoxP3DV1 $0
   ${If} $0 == ${BST_CHECKED}
      StrCpy $OAC_InstallPluginP3DV1 yes
   ${Else}
      StrCpy $OAC_InstallPluginP3DV1 no
   ${EndIf}

   ${NSD_GetState} $CheckBoxP3DV2 $0
   ${If} $0 == ${BST_CHECKED}
      StrCpy $OAC_InstallPluginP3DV2 yes
   ${Else}
      StrCpy $OAC_InstallPluginP3DV2 no
   ${EndIf}

   ${NSD_GetState} $CheckBoxP3DV3 $0
   ${If} $0 == ${BST_CHECKED}
      StrCpy $OAC_InstallPluginP3DV3 yes
   ${Else}
      StrCpy $OAC_InstallPluginP3DV3 no
   ${EndIf}
FunctionEnd

###
# Sections
###
Section "-Write Uninstaller"
   SetOutPath "$INSTDIR"
   WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "FlightVars Plugin" SecFlightVars
   SetOutPath "$INSTDIR\Modules"
   File "..\target\release\FlightVars.dll"

   ${OAC::EnablePlugin} FlightVars
SectionEnd

Section "un.FlightVars Plugin"
   ${OAC::DisablePlugin} "FlightVars"
SectionEnd

Section "Uninstall"
   RMDir /r "$INSTDIR"
   DeleteRegKey /ifempty HKCU "Software\FlightVars"
SectionEnd
